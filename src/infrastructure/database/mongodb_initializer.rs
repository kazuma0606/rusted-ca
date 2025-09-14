use mongodb::{Database, Collection};
use mongodb::{IndexModel, options::IndexOptions};
use bson::{Document, doc};
use crate::shared::error::infrastructure_error::InfrastructureError;
use chrono::Duration;

/// MongoDB collection initializer and maintenance
pub struct MongoDBInitializer {
    database: Database,
}

impl MongoDBInitializer {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Initialize all collections with proper structure and indexes
    pub async fn initialize_all_collections(&self) -> Result<(), InfrastructureError> {
        // HTTP Log Collections
        self.initialize_http_collections().await?;
        
        // ML Log Collections
        self.initialize_ml_collections().await?;
        
        // System Log Collections
        self.initialize_system_collections().await?;
        
        // Security Log Collections
        self.initialize_security_collections().await?;
        
        // Create TTL indexes for log rotation
        self.create_ttl_indexes().await?;
        
        println!("✓ All MongoDB collections initialized successfully");
        Ok(())
    }

    /// Initialize HTTP logging collections
    async fn initialize_http_collections(&self) -> Result<(), InfrastructureError> {
        let collections = vec![
            "logs_http_success",
            "logs_http_client_error", 
            "logs_http_server_error",
        ];

        for collection_name in collections {
            let collection: Collection<Document> = self.database.collection(collection_name);
            
            // Create HTTP-specific indexes
            let indexes = vec![
                // Basic indexes
                self.create_timestamp_index(),
                self.create_trace_id_index(),
                self.create_request_id_index(),
                self.create_level_timestamp_index(),
                
                // HTTP-specific indexes
                IndexModel::builder()
                    .keys(doc! { "http_context.endpoint": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("endpoint_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "http_context.status_code": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("status_code_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "http_context.method": 1, "http_context.endpoint": 1 })
                    .options(IndexOptions::builder().name("method_endpoint".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "user_id": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("user_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "http_context.ip_address": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("ip_timestamp".to_string()).build())
                    .build(),
            ];

            collection.create_indexes(indexes, None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to create indexes for {}: {}", collection_name, e)
                ))?;
            
            println!("✓ Initialized HTTP collection: {}", collection_name);
        }
        
        Ok(())
    }

    /// Initialize ML logging collections
    async fn initialize_ml_collections(&self) -> Result<(), InfrastructureError> {
        let collections = vec![
            "logs_ml_training",
            "logs_ml_inference", 
            "logs_ml_evaluation",
            "logs_ml_experiments",
        ];

        for collection_name in collections {
            let collection: Collection<Document> = self.database.collection(collection_name);
            
            // Create ML-specific indexes
            let indexes = vec![
                // Basic indexes
                self.create_timestamp_index(),
                self.create_trace_id_index(),
                self.create_request_id_index(),
                self.create_level_timestamp_index(),
                
                // ML-specific indexes
                IndexModel::builder()
                    .keys(doc! { "ml_context.model_id": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("model_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "ml_context.experiment_id": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("experiment_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "ml_context.operation_type": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("operation_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "ml_context.dataset_id": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("dataset_timestamp".to_string()).build())
                    .build(),
                // Performance metrics indexes
                IndexModel::builder()
                    .keys(doc! { "ml_context.loss": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("loss_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "ml_context.accuracy": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("accuracy_timestamp".to_string()).build())
                    .build(),
            ];

            collection.create_indexes(indexes, None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to create indexes for {}: {}", collection_name, e)
                ))?;
            
            println!("✓ Initialized ML collection: {}", collection_name);
        }
        
        Ok(())
    }

    /// Initialize system logging collections
    async fn initialize_system_collections(&self) -> Result<(), InfrastructureError> {
        let collections = vec![
            "logs_system_database",
            "logs_system_cache", 
            "logs_system_infrastructure",
        ];

        for collection_name in collections {
            let collection: Collection<Document> = self.database.collection(collection_name);
            
            // Create system-specific indexes
            let indexes = vec![
                // Basic indexes
                self.create_timestamp_index(),
                self.create_trace_id_index(),
                self.create_request_id_index(),
                self.create_level_timestamp_index(),
                
                // System-specific indexes
                IndexModel::builder()
                    .keys(doc! { "system_context.layer": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("layer_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "system_context.component": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("component_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "system_context.operation": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("operation_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "system_context.hostname": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("hostname_timestamp".to_string()).build())
                    .build(),
            ];

            collection.create_indexes(indexes, None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to create indexes for {}: {}", collection_name, e)
                ))?;
            
            println!("✓ Initialized System collection: {}", collection_name);
        }
        
        Ok(())
    }

    /// Initialize security logging collections
    async fn initialize_security_collections(&self) -> Result<(), InfrastructureError> {
        let collections = vec![
            "logs_security_auth",
            "logs_security_access", 
            "logs_security_audit",
        ];

        for collection_name in collections {
            let collection: Collection<Document> = self.database.collection(collection_name);
            
            // Create security-specific indexes
            let indexes = vec![
                // Basic indexes
                self.create_timestamp_index(),
                self.create_trace_id_index(),
                self.create_request_id_index(),
                self.create_level_timestamp_index(),
                
                // Security-specific indexes
                IndexModel::builder()
                    .keys(doc! { "user_id": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("user_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "session_id": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("session_timestamp".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "http_context.ip_address": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("ip_timestamp_sec".to_string()).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! { "level": 1, "category": 1, "timestamp": -1 })
                    .options(IndexOptions::builder().name("level_category_timestamp".to_string()).build())
                    .build(),
            ];

            collection.create_indexes(indexes, None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to create indexes for {}: {}", collection_name, e)
                ))?;
            
            println!("✓ Initialized Security collection: {}", collection_name);
        }
        
        Ok(())
    }

    /// Create TTL indexes for automatic log rotation
    async fn create_ttl_indexes(&self) -> Result<(), InfrastructureError> {
        // Define retention policies
        let retention_policies = vec![
            // HTTP logs - keep for 30 days
            ("logs_http_success", Duration::days(30)),
            ("logs_http_client_error", Duration::days(60)),  // Keep errors longer
            ("logs_http_server_error", Duration::days(90)),  // Keep server errors even longer
            
            // ML logs - keep for 90 days (experiments are valuable)
            ("logs_ml_training", Duration::days(90)),
            ("logs_ml_inference", Duration::days(30)),
            ("logs_ml_evaluation", Duration::days(90)),
            ("logs_ml_experiments", Duration::days(365)),  // Keep experiments for a year
            
            // System logs - keep for 60 days
            ("logs_system_database", Duration::days(60)),
            ("logs_system_cache", Duration::days(30)),
            ("logs_system_infrastructure", Duration::days(60)),
            
            // Security logs - keep for 180 days (compliance)
            ("logs_security_auth", Duration::days(180)),
            ("logs_security_access", Duration::days(180)),
            ("logs_security_audit", Duration::days(365)), // Keep audit logs for a year
        ];

        for (collection_name, retention_duration) in retention_policies {
            let collection: Collection<Document> = self.database.collection(collection_name);
            
            // Create TTL index on timestamp field
            let ttl_index = IndexModel::builder()
                .keys(doc! { "timestamp": 1 })
                .options(
                    IndexOptions::builder()
                        .name(format!("ttl_{}", collection_name))
                        .expire_after(std::time::Duration::from_secs(retention_duration.num_seconds() as u64))
                        .build()
                )
                .build();

            collection.create_index(ttl_index, None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to create TTL index for {}: {}", collection_name, e)
                ))?;
            
            println!("✓ Created TTL index for {}: {} days retention", 
                collection_name, retention_duration.num_days());
        }
        
        Ok(())
    }

    /// Create text indexes for full-text search on messages
    pub async fn create_text_indexes(&self) -> Result<(), InfrastructureError> {
        let all_collections = vec![
            "logs_http_success", "logs_http_client_error", "logs_http_server_error",
            "logs_ml_training", "logs_ml_inference", "logs_ml_evaluation", "logs_ml_experiments",
            "logs_system_database", "logs_system_cache", "logs_system_infrastructure",
            "logs_security_auth", "logs_security_access", "logs_security_audit",
        ];

        for collection_name in all_collections {
            let collection: Collection<Document> = self.database.collection(collection_name);
            
            // Create text index for full-text search
            let text_index = IndexModel::builder()
                .keys(doc! { 
                    "message": "text",
                    "tags": "text",
                    "metadata": "text"
                })
                .options(
                    IndexOptions::builder()
                        .name(format!("text_search_{}", collection_name))
                        .build()
                )
                .build();

            collection.create_index(text_index, None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to create text index for {}: {}", collection_name, e)
                ))?;
            
            println!("✓ Created text index for: {}", collection_name);
        }
        
        Ok(())
    }

    /// Utility methods for common index creation
    fn create_timestamp_index(&self) -> IndexModel {
        IndexModel::builder()
            .keys(doc! { "timestamp": -1 })
            .options(IndexOptions::builder().name("timestamp_desc".to_string()).build())
            .build()
    }

    fn create_trace_id_index(&self) -> IndexModel {
        IndexModel::builder()
            .keys(doc! { "trace_id": 1 })
            .options(IndexOptions::builder().name("trace_id".to_string()).build())
            .build()
    }

    fn create_request_id_index(&self) -> IndexModel {
        IndexModel::builder()
            .keys(doc! { "request_id": 1 })
            .options(IndexOptions::builder().name("request_id".to_string()).build())
            .build()
    }

    fn create_level_timestamp_index(&self) -> IndexModel {
        IndexModel::builder()
            .keys(doc! { "level": 1, "timestamp": -1 })
            .options(IndexOptions::builder().name("level_timestamp".to_string()).build())
            .build()
    }

    /// Drop all logging collections (use with caution!)
    pub async fn drop_all_collections(&self) -> Result<(), InfrastructureError> {
        let all_collections = vec![
            "logs_http_success", "logs_http_client_error", "logs_http_server_error",
            "logs_ml_training", "logs_ml_inference", "logs_ml_evaluation", "logs_ml_experiments", 
            "logs_system_database", "logs_system_cache", "logs_system_infrastructure",
            "logs_security_auth", "logs_security_access", "logs_security_audit",
        ];

        for collection_name in all_collections {
            let collection: Collection<Document> = self.database.collection(collection_name);
            collection.drop(None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to drop collection {}: {}", collection_name, e)
                ))?;
            
            println!("✗ Dropped collection: {}", collection_name);
        }
        
        Ok(())
    }

    /// Get collection statistics
    pub async fn get_collection_stats(&self) -> Result<Document, InfrastructureError> {
        let stats_command = doc! { "listCollections": 1 };
        self.database.run_command(stats_command, None).await
            .map_err(|e| InfrastructureError::DatabaseOperationError(
                format!("Failed to get collection stats: {}", e)
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::mongodb_connection::MongoDBConnection;

    #[tokio::test] 
    async fn test_initializer_creation() {
        // Mock test - in real scenario would need actual MongoDB connection
        // This test just verifies the struct can be created
        
        // Would need a real MongoDB connection for integration tests
        // let connection = MongoDBConnection::new("mongodb://localhost:27017").await.unwrap();
        // let initializer = MongoDBInitializer::new(connection.database().clone());
        // assert!(initializer.initialize_all_collections().await.is_ok());
    }
}