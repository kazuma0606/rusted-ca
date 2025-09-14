use std::sync::Arc;
use mongodb::{Collection, Database};
use bson::{Document, oid::ObjectId};

use crate::domain::entity::log_entry::LogEntry;
use crate::infrastructure::database::mongodb_connection::MongoDBConnection;
use crate::shared::error::infrastructure_error::InfrastructureError;

/// Repository for Security log operations
#[derive(Debug, Clone)]
pub struct SecurityLogRepository {
    auth_collection: Collection<Document>,
    access_collection: Collection<Document>,
    audit_collection: Collection<Document>,
    database: Database,
}

impl SecurityLogRepository {
    /// Create new Security log repository
    pub fn new(mongodb_connection: Arc<MongoDBConnection>) -> Self {
        let database = mongodb_connection.database().clone();
        
        Self {
            auth_collection: database.collection("logs_security_auth"),
            access_collection: database.collection("logs_security_access"),
            audit_collection: database.collection("logs_security_audit"),
            database,
        }
    }

    /// Insert Security log entry
    pub async fn insert_log(&self, log_entry: &LogEntry) -> Result<ObjectId, InfrastructureError> {
        let collection = match log_entry.get_collection_name().as_str() {
            "logs_security_auth" => &self.auth_collection,
            "logs_security_access" => &self.access_collection,
            "logs_security_audit" => &self.audit_collection,
            _ => &self.access_collection, // Default
        };
        
        let document = bson::to_document(log_entry)
            .map_err(|e| InfrastructureError::SerializationError(
                format!("Failed to serialize Security log entry: {}", e)
            ))?;

        let result = collection.insert_one(document, None).await
            .map_err(|e| InfrastructureError::DatabaseOperationError(
                format!("Failed to insert Security log: {}", e)
            ))?;

        Ok(result.inserted_id.as_object_id().unwrap())
    }

    /// Insert multiple Security log entries in batch
    pub async fn insert_many(&self, log_entries: Vec<&LogEntry>) -> Result<Vec<ObjectId>, InfrastructureError> {
        let mut all_ids = Vec::new();

        // Group by collection type
        let mut auth_logs = Vec::new();
        let mut access_logs = Vec::new();
        let mut audit_logs = Vec::new();

        for log_entry in log_entries {
            match log_entry.get_collection_name().as_str() {
                "logs_security_auth" => auth_logs.push(log_entry),
                "logs_security_access" => access_logs.push(log_entry),
                "logs_security_audit" => audit_logs.push(log_entry),
                _ => access_logs.push(log_entry), // Default
            }
        }

        // Insert into each collection
        if !auth_logs.is_empty() {
            let ids = self.batch_insert(&self.auth_collection, auth_logs).await?;
            all_ids.extend(ids);
        }

        if !access_logs.is_empty() {
            let ids = self.batch_insert(&self.access_collection, access_logs).await?;
            all_ids.extend(ids);
        }

        if !audit_logs.is_empty() {
            let ids = self.batch_insert(&self.audit_collection, audit_logs).await?;
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
                format!("Failed to batch insert Security logs: {}", e)
            ))?;

        let ids: Vec<ObjectId> = result.inserted_ids.values()
            .filter_map(|id| id.as_object_id())
            .collect();

        Ok(ids)
    }
}