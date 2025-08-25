use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, to_bson},
    options::{FindOptions, IndexOptions},
    Client, Collection, IndexModel,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};

use crate::domain::{
    entity::log_entry::LogEntry,
    repository::log_repository::{
        LogRepositoryInterface, LogSearchCriteria, LogSearchResult, PerformanceMetrics,
    },
    value_object::{analysis_status::AnalysisStatus, architecture_layer::ArchitectureLayer, log_id::LogId, request_id::RequestId},
};
use crate::infrastructure::logging::log_entry_document::LogEntryDocument;
use crate::shared::error::{domain_error::DomainError, infrastructure_error::InfrastructureError};

#[derive(Debug, Clone)]
pub struct BufferConfig {
    pub max_buffer_size: usize,
    pub flush_interval_ms: u64,
    pub max_retry_attempts: usize,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1000,
            flush_interval_ms: 5000, // 5 seconds
            max_retry_attempts: 3,
        }
    }
}

pub struct MongoDbLogRepository {
    collection: Collection<LogEntryDocument>,
    buffer: Arc<Mutex<Vec<LogEntryDocument>>>,
    buffer_config: BufferConfig,
    _flush_handle: Option<JoinHandle<()>>,
}

impl MongoDbLogRepository {
    pub async fn new(
        client: Client,
        database_name: &str,
        collection_name: &str,
    ) -> Result<Self, InfrastructureError> {
        let db = client.database(database_name);
        let collection = db.collection::<LogEntryDocument>(collection_name);

        // Create indexes for optimal query performance
        Self::create_indexes(&collection).await?;

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_config = BufferConfig::default();

        let mut repository = Self {
            collection,
            buffer,
            buffer_config,
            _flush_handle: None,
        };

        // Start background flush task
        repository.start_background_flush().await;

        Ok(repository)
    }

    async fn create_indexes(
        collection: &Collection<LogEntryDocument>,
    ) -> Result<(), InfrastructureError> {
        let indexes = vec![
            // Timestamp descending for chronological queries
            IndexModel::builder()
                .keys(doc! { "timestamp": -1 })
                .options(
                    IndexOptions::builder()
                        .name("timestamp_desc".to_string())
                        .build(),
                )
                .build(),
            // Compound index for level and layer filtering
            IndexModel::builder()
                .keys(doc! { "level": 1, "architecture_layer": 1 })
                .options(
                    IndexOptions::builder()
                        .name("level_layer".to_string())
                        .build(),
                )
                .build(),
            // Request ID for tracing
            IndexModel::builder()
                .keys(doc! { "request_id": 1 })
                .options(
                    IndexOptions::builder()
                        .name("request_id".to_string())
                        .build(),
                )
                .build(),
            // Log ID unique index
            IndexModel::builder()
                .keys(doc! { "log_id": 1 })
                .options(
                    IndexOptions::builder()
                        .name("log_id_unique".to_string())
                        .unique(true)
                        .build(),
                )
                .build(),
            // Status code for HTTP monitoring
            IndexModel::builder()
                .keys(doc! { "http_context.status_code": 1 })
                .options(
                    IndexOptions::builder()
                        .name("status_code".to_string())
                        .build(),
                )
                .build(),
            // Analysis status
            IndexModel::builder()
                .keys(doc! { "analysis_status": 1 })
                .options(
                    IndexOptions::builder()
                        .name("analysis_status".to_string())
                        .build(),
                )
                .build(),
        ];

        collection
            .create_indexes(indexes, None)
            .await
            .map_err(|e| InfrastructureError::DatabaseConnection { message: format!("Failed to create indexes: {}", e) })?;

        Ok(())
    }

    async fn start_background_flush(&mut self) {
        let collection = self.collection.clone();
        let buffer = self.buffer.clone();
        let flush_interval = Duration::from_millis(self.buffer_config.flush_interval_ms);

        let handle = tokio::spawn(async move {
            let mut interval = time::interval(flush_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::flush_buffer_internal(&collection, &buffer).await {
                    eprintln!("Background flush failed: {:?}", e);
                }
            }
        });

        self._flush_handle = Some(handle);
    }

    async fn flush_buffer_internal(
        collection: &Collection<LogEntryDocument>,
        buffer: &Arc<Mutex<Vec<LogEntryDocument>>>,
    ) -> Result<(), InfrastructureError> {
        let documents = {
            let mut buffer_guard = buffer.lock().await;
            if buffer_guard.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *buffer_guard)
        };

        if !documents.is_empty() {
            collection
                .insert_many(documents, None)
                .await
                .map_err(|e| InfrastructureError::DatabaseOperation(format!("Batch insert failed: {}", e)))?;
        }

        Ok(())
    }

    pub async fn flush_buffer(&self) -> Result<(), InfrastructureError> {
        Self::flush_buffer_internal(&self.collection, &self.buffer).await
    }
}

#[async_trait]
impl LogRepositoryInterface for MongoDbLogRepository {
    async fn store(&self, entry: &LogEntry) -> Result<(), DomainError> {
        let document = LogEntryDocument::from(entry.clone());

        // Add to buffer for batch processing
        {
            let mut buffer = self.buffer.lock().await;
            buffer.push(document);
        }

        // Check if immediate flush is needed
        let buffer_size = {
            let buffer = self.buffer.lock().await;
            buffer.len()
        };

        if buffer_size >= self.buffer_config.max_buffer_size {
            self.flush_buffer()
                .await
                .map_err(|e| DomainError::InvalidOperation(format!("Failed to flush buffer: {}", e)))?;
        }

        Ok(())
    }

    async fn find_by_id(&self, id: &LogId) -> Result<Option<LogEntry>, DomainError> {
        let filter = doc! { "log_id": id.to_string() };

        match self.collection.find_one(filter, None).await {
            Ok(Some(doc)) => {
                let entry = LogEntry::try_from(doc)
                    .map_err(|e| DomainError::InvalidValue(format!("Failed to deserialize log entry: {}", e)))?;
                Ok(Some(entry))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(DomainError::InvalidOperation(format!("Database query failed: {}", e))),
        }
    }

    async fn find_by_criteria(&self, criteria: &LogSearchCriteria) -> Result<LogSearchResult, DomainError> {
        criteria.validate()?;

        let filter = self.build_filter(criteria);
        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(criteria.limit.unwrap_or(100) as i64)
            .skip(criteria.offset.unwrap_or(0) as u64)
            .build();

        let mut cursor = self
            .collection
            .find(filter, options)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Database query failed: {}", e)))?;

        let mut logs = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Failed to read cursor: {}", e)))?
        {
            let entry = LogEntry::try_from(doc)
                .map_err(|e| DomainError::InvalidValue(format!("Failed to deserialize log entry: {}", e)))?;
            logs.push(entry);
        }

        Ok(LogSearchResult::simple(logs))
    }

    async fn update_analysis_status(&self, id: &LogId, status: AnalysisStatus) -> Result<(), DomainError> {
        let filter = doc! { "log_id": id.to_string() };
        let update = doc! {
            "$set": {
                "analysis_status": status.to_string(),
                "updated_at": to_bson(&Utc::now()).unwrap()
            }
        };

        self.collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Failed to update analysis status: {}", e)))?;

        Ok(())
    }

    async fn add_tags(&self, id: &LogId, tags: Vec<String>) -> Result<(), DomainError> {
        let filter = doc! { "log_id": id.to_string() };
        let update = doc! {
            "$addToSet": { "tags": { "$each": tags } },
            "$set": { "updated_at": to_bson(&Utc::now()).unwrap() }
        };

        self.collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Failed to add tags: {}", e)))?;

        Ok(())
    }

    async fn remove_tag(&self, id: &LogId, tag: &str) -> Result<(), DomainError> {
        let filter = doc! { "log_id": id.to_string() };
        let update = doc! {
            "$pull": { "tags": tag },
            "$set": { "updated_at": to_bson(&Utc::now()).unwrap() }
        };

        self.collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Failed to remove tag: {}", e)))?;

        Ok(())
    }

    async fn add_related_log(&self, id: &LogId, related_id: LogId) -> Result<(), DomainError> {
        let filter = doc! { "log_id": id.to_string() };
        let update = doc! {
            "$addToSet": { "related_log_ids": related_id.to_string() },
            "$set": { "updated_at": to_bson(&Utc::now()).unwrap() }
        };

        self.collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Failed to add related log: {}", e)))?;

        Ok(())
    }

    async fn count_by_criteria(&self, criteria: &LogSearchCriteria) -> Result<usize, DomainError> {
        criteria.validate()?;

        let filter = self.build_filter(criteria);
        let count = self
            .collection
            .count_documents(filter, None)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Count query failed: {}", e)))?;

        Ok(count as usize)
    }

    async fn delete_older_than(&self, cutoff_time: DateTime<Utc>) -> Result<usize, DomainError> {
        let filter = doc! { "timestamp": { "$lt": to_bson(&cutoff_time).unwrap() } };
        let result = self
            .collection
            .delete_many(filter, None)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Delete operation failed: {}", e)))?;

        Ok(result.deleted_count as usize)
    }

    async fn find_by_request_id(&self, request_id: &RequestId) -> Result<Vec<LogEntry>, DomainError> {
        let filter = doc! { "request_id": request_id.to_string() };
        let options = FindOptions::builder()
            .sort(doc! { "timestamp": 1 }) // Chronological order for tracing
            .build();

        let mut cursor = self
            .collection
            .find(filter, options)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Database query failed: {}", e)))?;

        let mut logs = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Failed to read cursor: {}", e)))?
        {
            let entry = LogEntry::try_from(doc)
                .map_err(|e| DomainError::InvalidValue(format!("Failed to deserialize log entry: {}", e)))?;
            logs.push(entry);
        }

        Ok(logs)
    }

    async fn find_errors_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogEntry>, DomainError> {
        let filter = doc! {
            "timestamp": { "$gte": to_bson(&start).unwrap(), "$lte": to_bson(&end).unwrap() },
            "level": { "$in": ["ERROR", "CRITICAL"] }
        };

        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(1000) // Limit for performance
            .build();

        let mut cursor = self
            .collection
            .find(filter, options)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Database query failed: {}", e)))?;

        let mut logs = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Failed to read cursor: {}", e)))?
        {
            let entry = LogEntry::try_from(doc)
                .map_err(|e| DomainError::InvalidValue(format!("Failed to deserialize log entry: {}", e)))?;
            logs.push(entry);
        }

        Ok(logs)
    }

    async fn get_performance_metrics(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<PerformanceMetrics, DomainError> {
        // This would typically use MongoDB aggregation pipeline
        // For now, implementing basic metrics calculation
        let filter = doc! {
            "timestamp": { "$gte": to_bson(&start).unwrap(), "$lte": to_bson(&end).unwrap() },
            "architecture_layer": "PRESENTATION" // HTTP requests only
        };

        let mut cursor = self
            .collection
            .find(filter, None)
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Database query failed: {}", e)))?;

        let mut metrics = PerformanceMetrics::new();
        let mut response_times = Vec::new();
        let mut error_count = 0;

        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e| DomainError::InvalidOperation(format!("Failed to read cursor: {}", e)))?
        {
            metrics.total_requests += 1;
            
            let response_time = doc.http_context.response_time_ms;
            response_times.push(response_time);

            let status_code = doc.http_context.status_code;
            *metrics.requests_by_status_code.entry(status_code).or_insert(0) += 1;

            if status_code >= 400 {
                error_count += 1;
            }

            if let Ok(layer) = ArchitectureLayer::from_string(&doc.architecture_layer) {
                *metrics.requests_by_layer.entry(layer).or_insert(0) += 1;
            }
        }

        if !response_times.is_empty() {
            response_times.sort();
            metrics.min_response_time_ms = response_times[0];
            metrics.max_response_time_ms = response_times[response_times.len() - 1];
            metrics.avg_response_time_ms = response_times.iter().sum::<u64>() as f64 / response_times.len() as f64;
            
            // Calculate P95
            let p95_index = (response_times.len() as f64 * 0.95) as usize;
            metrics.p95_response_time_ms = response_times[p95_index.min(response_times.len() - 1)];
        }

        if metrics.total_requests > 0 {
            metrics.error_rate = error_count as f64 / metrics.total_requests as f64;
        }

        Ok(metrics)
    }
}

impl MongoDbLogRepository {
    fn build_filter(&self, criteria: &LogSearchCriteria) -> mongodb::bson::Document {
        let mut filter = doc! {};

        if let Some(level) = &criteria.level {
            filter.insert("level", level.to_string());
        }

        if let Some(layer) = &criteria.architecture_layer {
            filter.insert("architecture_layer", layer.to_string());
        }

        if let Some(request_id) = &criteria.request_id {
            filter.insert("request_id", request_id.to_string());
        }

        if let Some(user_id) = &criteria.user_id {
            filter.insert("user_context.user_id", user_id);
        }

        if let (Some(start), Some(end)) = (&criteria.start_time, &criteria.end_time) {
            filter.insert(
                "timestamp", 
                doc! { 
                    "$gte": to_bson(start).unwrap(), 
                    "$lte": to_bson(end).unwrap() 
                }
            );
        } else if let Some(start) = &criteria.start_time {
            filter.insert("timestamp", doc! { "$gte": to_bson(start).unwrap() });
        } else if let Some(end) = &criteria.end_time {
            filter.insert("timestamp", doc! { "$lte": to_bson(end).unwrap() });
        }

        if let Some(message_text) = &criteria.message_contains {
            filter.insert("message", doc! { "$regex": message_text, "$options": "i" });
        }

        if let Some(tags) = &criteria.tags {
            if !tags.is_empty() {
                filter.insert("tags", doc! { "$in": tags });
            }
        }

        if let Some(status) = &criteria.analysis_status {
            filter.insert("analysis_status", status.to_string());
        }

        filter
    }
}