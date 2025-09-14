use std::sync::Arc;
use mongodb::{Collection, Database};
use bson::{Document, oid::ObjectId};

use crate::domain::entity::log_entry::LogEntry;
use crate::infrastructure::database::mongodb_connection::MongoDBConnection;
use crate::shared::error::infrastructure_error::InfrastructureError;

/// Repository for Machine Learning log operations
#[derive(Debug, Clone)]
pub struct MLLogRepository {
    training_collection: Collection<Document>,
    inference_collection: Collection<Document>,
    evaluation_collection: Collection<Document>,
    experiments_collection: Collection<Document>,
    database: Database,
}

impl MLLogRepository {
    /// Create new ML log repository
    pub fn new(mongodb_connection: Arc<MongoDBConnection>) -> Self {
        let database = mongodb_connection.database().clone();
        
        Self {
            training_collection: database.collection("logs_ml_training"),
            inference_collection: database.collection("logs_ml_inference"),
            evaluation_collection: database.collection("logs_ml_evaluation"),
            experiments_collection: database.collection("logs_ml_experiments"),
            database,
        }
    }

    /// Insert ML log entry
    pub async fn insert_log(&self, log_entry: &LogEntry) -> Result<ObjectId, InfrastructureError> {
        let collection = match log_entry.get_collection_name().as_str() {
            "logs_ml_training" => &self.training_collection,
            "logs_ml_inference" => &self.inference_collection,
            "logs_ml_evaluation" => &self.evaluation_collection,
            "logs_ml_experiments" => &self.experiments_collection,
            _ => &self.training_collection, // Default
        };
        
        let document = bson::to_document(log_entry)
            .map_err(|e| InfrastructureError::SerializationError(
                format!("Failed to serialize ML log entry: {}", e)
            ))?;

        let result = collection.insert_one(document, None).await
            .map_err(|e| InfrastructureError::DatabaseOperationError(
                format!("Failed to insert ML log: {}", e)
            ))?;

        Ok(result.inserted_id.as_object_id().unwrap())
    }

    /// Insert multiple ML log entries in batch
    pub async fn insert_many(&self, log_entries: Vec<&LogEntry>) -> Result<Vec<ObjectId>, InfrastructureError> {
        let mut all_ids = Vec::new();

        // Group by collection type
        let mut training_logs = Vec::new();
        let mut inference_logs = Vec::new();
        let mut evaluation_logs = Vec::new();
        let mut experiment_logs = Vec::new();

        for log_entry in log_entries {
            match log_entry.get_collection_name().as_str() {
                "logs_ml_training" => training_logs.push(log_entry),
                "logs_ml_inference" => inference_logs.push(log_entry),
                "logs_ml_evaluation" => evaluation_logs.push(log_entry),
                "logs_ml_experiments" => experiment_logs.push(log_entry),
                _ => training_logs.push(log_entry), // Default
            }
        }

        // Insert into each collection
        if !training_logs.is_empty() {
            let ids = self.batch_insert(&self.training_collection, training_logs).await?;
            all_ids.extend(ids);
        }

        if !inference_logs.is_empty() {
            let ids = self.batch_insert(&self.inference_collection, inference_logs).await?;
            all_ids.extend(ids);
        }

        if !evaluation_logs.is_empty() {
            let ids = self.batch_insert(&self.evaluation_collection, evaluation_logs).await?;
            all_ids.extend(ids);
        }

        if !experiment_logs.is_empty() {
            let ids = self.batch_insert(&self.experiments_collection, experiment_logs).await?;
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
                format!("Failed to batch insert ML logs: {}", e)
            ))?;

        let ids: Vec<ObjectId> = result.inserted_ids.values()
            .filter_map(|id| id.as_object_id())
            .collect();

        Ok(ids)
    }
}