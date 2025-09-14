use std::sync::Arc;
use std::collections::HashMap;
use mongodb::{Collection, Database};
use mongodb::options::FindOptions;
use bson::{Document, doc};
use chrono::{DateTime, Utc, Duration};
use futures::stream::StreamExt;

use crate::domain::entity::log_entry::{LogEntry, LogCategory};
use crate::infrastructure::database::mongodb_connection::MongoDBConnection;
use crate::shared::error::infrastructure_error::InfrastructureError;

/// Repository for querying logs across all collections
#[derive(Debug, Clone)]
pub struct LogQueryRepository {
    collections: HashMap<LogCategory, Vec<Collection<Document>>>,
    database: Database,
}

impl LogQueryRepository {
    /// Create new Log Query repository
    pub fn new(mongodb_connection: Arc<MongoDBConnection>) -> Self {
        let database = mongodb_connection.database().clone();
        
        let mut collections = HashMap::new();

        // HTTP collections
        collections.insert(LogCategory::Http, vec![
            database.collection("logs_http_success"),
            database.collection("logs_http_client_error"),
            database.collection("logs_http_server_error"),
        ]);

        // ML collections
        collections.insert(LogCategory::ML, vec![
            database.collection("logs_ml_training"),
            database.collection("logs_ml_inference"),
            database.collection("logs_ml_evaluation"),
            database.collection("logs_ml_experiments"),
        ]);

        // System collections
        collections.insert(LogCategory::System, vec![
            database.collection("logs_system_database"),
            database.collection("logs_system_cache"),
            database.collection("logs_system_infrastructure"),
        ]);

        // Security collections
        collections.insert(LogCategory::Security, vec![
            database.collection("logs_security_auth"),
            database.collection("logs_security_access"),
            database.collection("logs_security_audit"),
        ]);

        Self {
            collections,
            database,
        }
    }

    /// Find logs by trace ID across all collections
    pub async fn find_by_trace_id(&self, trace_id: &str) -> Result<Vec<LogEntry>, InfrastructureError> {
        let filter = doc! { "trace_id": trace_id };
        let mut logs = Vec::new();

        // Search in all collections
        for collections_vec in self.collections.values() {
            for collection in collections_vec {
                let mut cursor = collection.find(filter.clone(), None).await
                    .map_err(|e| InfrastructureError::DatabaseOperationError(
                        format!("Failed to query logs by trace_id: {}", e)
                    ))?;

                while let Some(doc) = cursor.next().await {
                    let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                        format!("Failed to iterate log cursor: {}", e)
                    ))?;

                    let log_entry: LogEntry = bson::from_document(doc)
                        .map_err(|e| InfrastructureError::DeserializationError(
                            format!("Failed to deserialize log entry: {}", e)
                        ))?;

                    logs.push(log_entry);
                }
            }
        }

        // Sort by timestamp
        logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(logs)
    }

    /// Find logs by request ID across all collections
    pub async fn find_by_request_id(&self, request_id: &str) -> Result<Vec<LogEntry>, InfrastructureError> {
        let filter = doc! { "request_id": request_id };
        let mut logs = Vec::new();

        // Search in all collections
        for collections_vec in self.collections.values() {
            for collection in collections_vec {
                let mut cursor = collection.find(filter.clone(), None).await
                    .map_err(|e| InfrastructureError::DatabaseOperationError(
                        format!("Failed to query logs by request_id: {}", e)
                    ))?;

                while let Some(doc) = cursor.next().await {
                    let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                        format!("Failed to iterate log cursor: {}", e)
                    ))?;

                    let log_entry: LogEntry = bson::from_document(doc)
                        .map_err(|e| InfrastructureError::DeserializationError(
                            format!("Failed to deserialize log entry: {}", e)
                        ))?;

                    logs.push(log_entry);
                }
            }
        }

        // Sort by timestamp
        logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(logs)
    }

    /// Find logs by category and time range
    pub async fn find_by_category_and_time(
        &self,
        category: LogCategory,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: Option<i64>,
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

        if let Some(collections_vec) = self.collections.get(&category) {
            for collection in collections_vec {
                let mut cursor = collection.find(filter.clone(), options.clone()).await
                    .map_err(|e| InfrastructureError::DatabaseOperationError(
                        format!("Failed to query logs by category and time: {}", e)
                    ))?;

                while let Some(doc) = cursor.next().await {
                    let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                        format!("Failed to iterate log cursor: {}", e)
                    ))?;

                    let log_entry: LogEntry = bson::from_document(doc)
                        .map_err(|e| InfrastructureError::DeserializationError(
                            format!("Failed to deserialize log entry: {}", e)
                        ))?;

                    logs.push(log_entry);
                }
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

    /// Get error logs within time range across all collections
    pub async fn get_errors_in_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: Option<i64>,
    ) -> Result<Vec<LogEntry>, InfrastructureError> {
        let filter = doc! {
            "level": { "$in": ["Error", "Fatal"] },
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
        for collections_vec in self.collections.values() {
            for collection in collections_vec {
                let mut cursor = collection.find(filter.clone(), options.clone()).await
                    .map_err(|e| InfrastructureError::DatabaseOperationError(
                        format!("Failed to query error logs: {}", e)
                    ))?;

                while let Some(doc) = cursor.next().await {
                    let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                        format!("Failed to iterate error log cursor: {}", e)
                    ))?;

                    let log_entry: LogEntry = bson::from_document(doc)
                        .map_err(|e| InfrastructureError::DeserializationError(
                            format!("Failed to deserialize error log entry: {}", e)
                        ))?;

                    logs.push(log_entry);
                }
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

    /// Execute custom aggregation pipeline on a specific collection
    pub async fn aggregate_collection(
        &self,
        collection_name: &str,
        pipeline: Vec<Document>,
    ) -> Result<Vec<Document>, InfrastructureError> {
        let collection: Collection<Document> = self.database.collection(collection_name);
        
        let mut cursor = collection.aggregate(pipeline, None).await
            .map_err(|e| InfrastructureError::DatabaseOperationError(
                format!("Failed to execute aggregation on {}: {}", collection_name, e)
            ))?;

        let mut results = Vec::new();
        while let Some(doc) = cursor.next().await {
            let doc = doc.map_err(|e| InfrastructureError::DatabaseOperationError(
                format!("Failed to iterate aggregation results: {}", e)
            ))?;
            results.push(doc);
        }

        Ok(results)
    }

    /// Get log statistics for a specific category
    pub async fn get_category_stats(&self, category: LogCategory) -> Result<CategoryStats, InfrastructureError> {
        let mut total_logs = 0i64;
        let mut collection_counts = HashMap::new();

        if let Some(collections_vec) = self.collections.get(&category) {
            for collection in collections_vec {
                let count = collection.count_documents(doc! {}, None).await
                    .map_err(|e| InfrastructureError::DatabaseOperationError(
                        format!("Failed to count documents: {}", e)
                    ))?;
                
                total_logs += count as i64;
                collection_counts.insert(
                    collection.name().to_string(), 
                    count as u64
                );
            }
        }

        Ok(CategoryStats {
            category,
            total_logs,
            collection_counts,
            last_updated: Utc::now(),
        })
    }
}

/// Statistics for a log category
#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: LogCategory,
    pub total_logs: i64,
    pub collection_counts: HashMap<String, u64>,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_stats_creation() {
        let mut collection_counts = HashMap::new();
        collection_counts.insert("logs_http_success".to_string(), 100);

        let stats = CategoryStats {
            category: LogCategory::Http,
            total_logs: 100,
            collection_counts,
            last_updated: Utc::now(),
        };

        assert_eq!(stats.category, LogCategory::Http);
        assert_eq!(stats.total_logs, 100);
    }
}