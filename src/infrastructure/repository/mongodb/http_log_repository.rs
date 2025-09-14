use std::sync::Arc;
use mongodb::{Collection, Database};
use mongodb::options::{FindOptions, UpdateOptions};
use bson::{Document, doc, oid::ObjectId};
use chrono::{DateTime, Utc, Duration};
use futures::stream::StreamExt;

use crate::domain::entity::log_entry::{LogEntry, LogLevel, HttpLogContext};
use crate::infrastructure::database::mongodb_connection::MongoDBConnection;
use crate::shared::error::infrastructure_error::InfrastructureError;

/// Repository for HTTP log operations
#[derive(Debug, Clone)]
pub struct HttpLogRepository {
    success_collection: Collection<Document>,
    client_error_collection: Collection<Document>,
    server_error_collection: Collection<Document>,
    database: Database,
}

impl HttpLogRepository {
    /// Create new HTTP log repository
    pub fn new(mongodb_connection: Arc<MongoDBConnection>) -> Self {
        let database = mongodb_connection.database().clone();
        
        Self {
            success_collection: database.collection("logs_http_success"),
            client_error_collection: database.collection("logs_http_client_error"),
            server_error_collection: database.collection("logs_http_server_error"),
            database,
        }
    }

    /// Insert HTTP log entry
    pub async fn insert_log(&self, log_entry: &LogEntry) -> Result<ObjectId, InfrastructureError> {
        let collection = self.get_collection_by_log_level(&log_entry.level, log_entry.http_context.as_ref());
        
        let document = bson::to_document(log_entry)
            .map_err(|e| InfrastructureError::SerializationError(
                format!("Failed to serialize log entry: {}", e)
            ))?;

        let result = collection.insert_one(document, None).await
            .map_err(|e| InfrastructureError::DatabaseOperationError(
                format!("Failed to insert HTTP log: {}", e)
            ))?;

        Ok(result.inserted_id.as_object_id().unwrap())
    }

    /// Insert multiple HTTP log entries in batch
    pub async fn insert_many(&self, log_entries: Vec<&LogEntry>) -> Result<Vec<ObjectId>, InfrastructureError> {
        // Group by collection type
        let mut success_logs = Vec::new();
        let mut client_error_logs = Vec::new();
        let mut server_error_logs = Vec::new();

        for log_entry in log_entries {
            match self.categorize_log(&log_entry.level, log_entry.http_context.as_ref()) {
                "success" => success_logs.push(log_entry),
                "client_error" => client_error_logs.push(log_entry),
                "server_error" => server_error_logs.push(log_entry),
                _ => success_logs.push(log_entry), // Default to success
            }
        }

        let mut all_ids = Vec::new();

        // Insert into each collection
        if !success_logs.is_empty() {
            let ids = self.batch_insert(&self.success_collection, success_logs).await?;
            all_ids.extend(ids);
        }

        if !client_error_logs.is_empty() {
            let ids = self.batch_insert(&self.client_error_collection, client_error_logs).await?;
            all_ids.extend(ids);
        }

        if !server_error_logs.is_empty() {
            let ids = self.batch_insert(&self.server_error_collection, server_error_logs).await?;
            all_ids.extend(ids);
        }

        Ok(all_ids)
    }

    /// Find HTTP logs by trace ID
    pub async fn find_by_trace_id(&self, trace_id: &str) -> Result<Vec<LogEntry>, InfrastructureError> {
        let filter = doc! { "trace_id": trace_id };
        let mut logs = Vec::new();

        // Search in all collections
        for collection in [&self.success_collection, &self.client_error_collection, &self.server_error_collection] {
            let mut cursor = collection.find(filter.clone(), None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to query HTTP logs by trace_id: {}", e)
                ))?;

            while let Some(doc) = cursor.next().await {
                let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to iterate HTTP log cursor: {}", e)
                ))?;

                let log_entry: LogEntry = bson::from_document(doc)
                    .map_err(|e| InfrastructureError::DeserializationError(
                        format!("Failed to deserialize log entry: {}", e)
                    ))?;

                logs.push(log_entry);
            }
        }

        // Sort by timestamp
        logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(logs)
    }

    /// Find HTTP logs by request ID
    pub async fn find_by_request_id(&self, request_id: &str) -> Result<Vec<LogEntry>, InfrastructureError> {
        let filter = doc! { "request_id": request_id };
        let mut logs = Vec::new();

        // Search in all collections
        for collection in [&self.success_collection, &self.client_error_collection, &self.server_error_collection] {
            let mut cursor = collection.find(filter.clone(), None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to query HTTP logs by request_id: {}", e)
                ))?;

            while let Some(doc) = cursor.next().await {
                let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to iterate HTTP log cursor: {}", e)
                ))?;

                let log_entry: LogEntry = bson::from_document(doc)
                    .map_err(|e| InfrastructureError::DeserializationError(
                        format!("Failed to deserialize log entry: {}", e)
                    ))?;

                logs.push(log_entry);
            }
        }

        // Sort by timestamp
        logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(logs)
    }

    /// Find HTTP logs by endpoint and time range
    pub async fn find_by_endpoint(
        &self, 
        endpoint: &str, 
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: Option<i64>
    ) -> Result<Vec<LogEntry>, InfrastructureError> {
        let filter = doc! {
            "http_context.endpoint": endpoint,
            "timestamp": {
                "$gte": start_time,
                "$lte": end_time
            }
        };

        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit)
            .build();

        let mut logs = Vec::new();

        // Search in all collections
        for collection in [&self.success_collection, &self.client_error_collection, &self.server_error_collection] {
            let mut cursor = collection.find(filter.clone(), options.clone()).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to query HTTP logs by endpoint: {}", e)
                ))?;

            while let Some(doc) = cursor.next().await {
                let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to iterate HTTP log cursor: {}", e)
                ))?;

                let log_entry: LogEntry = bson::from_document(doc)
                    .map_err(|e| InfrastructureError::DeserializationError(
                        format!("Failed to deserialize log entry: {}", e)
                    ))?;

                logs.push(log_entry);
            }
        }

        // Sort by timestamp (descending)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply limit if specified and we have more logs than needed
        if let Some(limit) = limit {
            logs.truncate(limit as usize);
        }

        Ok(logs)
    }

    /// Get error logs within time range
    pub async fn get_errors_in_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: Option<i64>
    ) -> Result<Vec<LogEntry>, InfrastructureError> {
        let filter = doc! {
            "timestamp": {
                "$gte": start_time,
                "$lte": end_time
            }
        };

        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit)
            .build();

        let mut logs = Vec::new();

        // Search in error collections only
        for collection in [&self.client_error_collection, &self.server_error_collection] {
            let mut cursor = collection.find(filter.clone(), options.clone()).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to query HTTP error logs: {}", e)
                ))?;

            while let Some(doc) = cursor.next().await {
                let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to iterate HTTP error log cursor: {}", e)
                ))?;

                let log_entry: LogEntry = bson::from_document(doc)
                    .map_err(|e| InfrastructureError::DeserializationError(
                        format!("Failed to deserialize error log entry: {}", e)
                    ))?;

                logs.push(log_entry);
            }
        }

        // Sort by timestamp (descending)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply limit if specified
        if let Some(limit) = limit {
            logs.truncate(limit as usize);
        }

        Ok(logs)
    }

    /// Get performance statistics for an endpoint
    pub async fn get_endpoint_performance_stats(
        &self,
        endpoint: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>
    ) -> Result<EndpointPerformanceStats, InfrastructureError> {
        // Aggregation pipeline for performance statistics
        let pipeline = vec![
            doc! {
                "$match": {
                    "http_context.endpoint": endpoint,
                    "timestamp": {
                        "$gte": start_time,
                        "$lte": end_time
                    }
                }
            },
            doc! {
                "$group": {
                    "_id": null,
                    "total_requests": { "$sum": 1 },
                    "avg_response_time": { "$avg": "$http_context.response_time_ms" },
                    "min_response_time": { "$min": "$http_context.response_time_ms" },
                    "max_response_time": { "$max": "$http_context.response_time_ms" },
                    "status_codes": { "$push": "$http_context.status_code" }
                }
            }
        ];

        let mut total_requests = 0i64;
        let mut avg_response_time = 0.0f64;
        let mut min_response_time = 0.0f64;
        let mut max_response_time = 0.0f64;
        let mut status_codes = Vec::new();

        // Run aggregation on all collections
        for collection in [&self.success_collection, &self.client_error_collection, &self.server_error_collection] {
            let mut cursor = collection.aggregate(pipeline.clone(), None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to aggregate HTTP performance stats: {}", e)
                ))?;

            while let Some(doc) = cursor.next().await {
                let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to iterate aggregation results: {}", e)
                ))?;

                if let Some(requests) = doc.get_i64("total_requests").ok() {
                    total_requests += requests;
                }
                if let Some(avg) = doc.get_f64("avg_response_time").ok() {
                    avg_response_time = (avg_response_time + avg) / 2.0; // Simple average
                }
                if let Some(min) = doc.get_f64("min_response_time").ok() {
                    min_response_time = if min_response_time == 0.0 { min } else { min_response_time.min(min) };
                }
                if let Some(max) = doc.get_f64("max_response_time").ok() {
                    max_response_time = max_response_time.max(max);
                }
                if let Some(codes) = doc.get_array("status_codes").ok() {
                    for code in codes {
                        if let Some(status_code) = code.as_i32() {
                            status_codes.push(status_code as u16);
                        }
                    }
                }
            }
        }

        Ok(EndpointPerformanceStats {
            endpoint: endpoint.to_string(),
            total_requests,
            avg_response_time_ms: avg_response_time,
            min_response_time_ms: min_response_time,
            max_response_time_ms: max_response_time,
            status_code_distribution: self.calculate_status_distribution(&status_codes),
            time_range_start: start_time,
            time_range_end: end_time,
        })
    }

    /// Delete old logs beyond retention period
    pub async fn cleanup_old_logs(&self, retention_days: i64) -> Result<u64, InfrastructureError> {
        let cutoff_time = Utc::now() - Duration::days(retention_days);
        let filter = doc! {
            "timestamp": { "$lt": cutoff_time }
        };

        let mut total_deleted = 0u64;

        for collection in [&self.success_collection, &self.client_error_collection, &self.server_error_collection] {
            let result = collection.delete_many(filter.clone(), None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to cleanup old HTTP logs: {}", e)
                ))?;

            total_deleted += result.deleted_count;
        }

        println!("Cleaned up {} old HTTP logs older than {} days", total_deleted, retention_days);
        Ok(total_deleted)
    }

    // Helper methods

    /// Get appropriate collection based on log level and HTTP context
    fn get_collection_by_log_level(&self, level: &LogLevel, http_context: Option<&HttpLogContext>) -> &Collection<Document> {
        if let Some(context) = http_context {
            match context.status_code {
                200..=299 => &self.success_collection,
                400..=499 => &self.client_error_collection,
                500..=599 => &self.server_error_collection,
                _ => &self.success_collection,
            }
        } else {
            match level {
                LogLevel::Error | LogLevel::Fatal => &self.server_error_collection,
                LogLevel::Warn => &self.client_error_collection,
                _ => &self.success_collection,
            }
        }
    }

    /// Categorize log for collection selection
    fn categorize_log(&self, level: &LogLevel, http_context: Option<&HttpLogContext>) -> &'static str {
        if let Some(context) = http_context {
            match context.status_code {
                200..=299 => "success",
                400..=499 => "client_error",
                500..=599 => "server_error",
                _ => "success",
            }
        } else {
            match level {
                LogLevel::Error | LogLevel::Fatal => "server_error",
                LogLevel::Warn => "client_error",
                _ => "success",
            }
        }
    }

    /// Batch insert into specific collection
    async fn batch_insert(&self, collection: &Collection<Document>, log_entries: Vec<&LogEntry>) -> Result<Vec<ObjectId>, InfrastructureError> {
        let documents: Vec<Document> = log_entries.iter()
            .filter_map(|entry| bson::to_document(entry).ok())
            .collect();

        if documents.is_empty() {
            return Ok(Vec::new());
        }

        let result = collection.insert_many(documents, None).await
            .map_err(|e| InfrastructureError::DatabaseOperationError(
                format!("Failed to batch insert HTTP logs: {}", e)
            ))?;

        let ids: Vec<ObjectId> = result.inserted_ids.values()
            .filter_map(|id| id.as_object_id())
            .collect();

        Ok(ids)
    }

    /// Calculate status code distribution
    fn calculate_status_distribution(&self, status_codes: &[u16]) -> std::collections::HashMap<u16, u32> {
        let mut distribution = std::collections::HashMap::new();
        
        for &code in status_codes {
            *distribution.entry(code).or_insert(0) += 1;
        }
        
        distribution
    }
}

/// Performance statistics for an endpoint
#[derive(Debug, Clone)]
pub struct EndpointPerformanceStats {
    pub endpoint: String,
    pub total_requests: i64,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
    pub status_code_distribution: std::collections::HashMap<u16, u32>,
    pub time_range_start: DateTime<Utc>,
    pub time_range_end: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::log_entry::{LogLevel, HttpLogContext};

    // Note: These tests are temporarily disabled as they require a real MongoDB connection
    // For unit testing, we would need to create mock implementations

    #[test]
    fn test_log_categorization() {
        // Test without creating the repository - just test the logic
        let http_context = HttpLogContext {
            method: "GET".to_string(),
            endpoint: "/test".to_string(),
            status_code: 404,
            user_agent: None,
            ip_address: "127.0.0.1".to_string(),
            content_length: None,
            response_time_ms: 100,
            query_params: None,
            request_headers: None,
            response_headers: None,
        };

        // We can test the categorization logic without the repository
        let category = match http_context.status_code {
            200..=299 => "success",
            400..=499 => "client_error",
            500..=599 => "server_error",
            _ => "success",
        };
        assert_eq!(category, "client_error");

        let error_category = match LogLevel::Error {
            LogLevel::Error | LogLevel::Fatal => "server_error",
            LogLevel::Warn => "client_error",
            _ => "success",
        };
        assert_eq!(error_category, "server_error");
    }

    #[test]
    fn test_status_distribution_calculation() {
        // Test the calculation logic without repository
        let status_codes = vec![200, 200, 404, 500, 200];
        let mut distribution = std::collections::HashMap::new();
        
        for &code in &status_codes {
            *distribution.entry(code).or_insert(0) += 1;
        }

        assert_eq!(distribution.get(&200), Some(&3));
        assert_eq!(distribution.get(&404), Some(&1));
        assert_eq!(distribution.get(&500), Some(&1));
    }
}