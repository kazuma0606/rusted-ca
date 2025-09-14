use std::sync::Arc;
use mongodb::{Collection, Database};
use bson::{Document, oid::ObjectId};

use crate::domain::entity::log_entry::LogEntry;
use crate::infrastructure::database::mongodb_connection::MongoDBConnection;
use crate::shared::error::infrastructure_error::InfrastructureError;

/// Repository for System log operations
#[derive(Debug, Clone)]
pub struct SystemLogRepository {
    database_collection: Collection<Document>,
    cache_collection: Collection<Document>,
    infrastructure_collection: Collection<Document>,
    database: Database,
}

impl SystemLogRepository {
    /// Create new System log repository
    pub fn new(mongodb_connection: Arc<MongoDBConnection>) -> Self {
        let database = mongodb_connection.database().clone();
        
        Self {
            database_collection: database.collection("logs_system_database"),
            cache_collection: database.collection("logs_system_cache"),
            infrastructure_collection: database.collection("logs_system_infrastructure"),
            database,
        }
    }

    /// Insert System log entry
    pub async fn insert_log(&self, log_entry: &LogEntry) -> Result<ObjectId, InfrastructureError> {
        let collection = match log_entry.get_collection_name().as_str() {
            "logs_system_database" => &self.database_collection,
            "logs_system_cache" => &self.cache_collection,
            "logs_system_infrastructure" => &self.infrastructure_collection,
            _ => &self.infrastructure_collection, // Default
        };
        
        let document = bson::to_document(log_entry)
            .map_err(|e| InfrastructureError::SerializationError(
                format!("Failed to serialize System log entry: {}", e)
            ))?;

        let result = collection.insert_one(document, None).await
            .map_err(|e| InfrastructureError::DatabaseOperationError(
                format!("Failed to insert System log: {}", e)
            ))?;

        Ok(result.inserted_id.as_object_id().unwrap())
    }

    /// Insert multiple System log entries in batch
    pub async fn insert_many(&self, log_entries: Vec<&LogEntry>) -> Result<Vec<ObjectId>, InfrastructureError> {
        let mut all_ids = Vec::new();

        // Group by collection type
        let mut database_logs = Vec::new();
        let mut cache_logs = Vec::new();
        let mut infrastructure_logs = Vec::new();

        for log_entry in log_entries {
            match log_entry.get_collection_name().as_str() {
                "logs_system_database" => database_logs.push(log_entry),
                "logs_system_cache" => cache_logs.push(log_entry),
                "logs_system_infrastructure" => infrastructure_logs.push(log_entry),
                _ => infrastructure_logs.push(log_entry), // Default
            }
        }

        // Insert into each collection
        if !database_logs.is_empty() {
            let ids = self.batch_insert(&self.database_collection, database_logs).await?;
            all_ids.extend(ids);
        }

        if !cache_logs.is_empty() {
            let ids = self.batch_insert(&self.cache_collection, cache_logs).await?;
            all_ids.extend(ids);
        }

        if !infrastructure_logs.is_empty() {
            let ids = self.batch_insert(&self.infrastructure_collection, infrastructure_logs).await?;
            all_ids.extend(ids);
        }

        Ok(all_ids)
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
                format!("Failed to batch insert System logs: {}", e)
            ))?;

        let ids: Vec<ObjectId> = result.inserted_ids.values()
            .filter_map(|id| id.as_object_id())
            .collect();

        Ok(ids)
    }
}