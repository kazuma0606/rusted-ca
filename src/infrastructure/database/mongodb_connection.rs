use mongodb::{Client, Database, Collection};
use mongodb::options::{ClientOptions, ResolverConfig};
use std::collections::HashMap;
use bson::Document;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::shared::error::infrastructure_error::InfrastructureError;

/// MongoDB connection and database management
#[derive(Debug, Clone)]
pub struct MongoDBConnection {
    client: Client,
    database: Database,
}

/// MongoDB collection manager for different log types
#[derive(Debug, Clone)]
pub struct MongoCollections {
    pub http_success: Collection<Document>,
    pub http_client_error: Collection<Document>,
    pub http_server_error: Collection<Document>,
    pub ml_training: Collection<Document>,
    pub ml_inference: Collection<Document>,
    pub ml_evaluation: Collection<Document>,
    pub ml_experiments: Collection<Document>,
    pub system_database: Collection<Document>,
    pub system_cache: Collection<Document>,
    pub system_infrastructure: Collection<Document>,
    pub security_auth: Collection<Document>,
    pub security_access: Collection<Document>,
    pub security_audit: Collection<Document>,
}

impl MongoDBConnection {
    /// Create a new MongoDB connection
    pub async fn new(connection_string: &str) -> Result<Self, InfrastructureError> {
        // Parse connection options
        let mut client_options = ClientOptions::parse_with_resolver_config(
            connection_string, 
            ResolverConfig::cloudflare()
        ).await.map_err(|e| InfrastructureError::DatabaseConnectionError(
            format!("Failed to parse MongoDB connection string: {}", e)
        ))?;

        // Set application name for monitoring
        client_options.app_name = Some("rusted-ca-logging".to_string());
        
        // Create client
        let client = Client::with_options(client_options)
            .map_err(|e| InfrastructureError::DatabaseConnectionError(
                format!("Failed to create MongoDB client: {}", e)
            ))?;

        // Get database
        let database = client.database("rusted_ca_logs");

        // Test connection
        client
            .database("admin")
            .run_command(bson::doc! { "ping": 1 }, None)
            .await
            .map_err(|e| InfrastructureError::DatabaseConnectionError(
                format!("Failed to connect to MongoDB: {}", e)
            ))?;

        Ok(Self {
            client,
            database,
        })
    }

    /// Get database reference
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Get client reference
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Initialize all collections with proper indexes
    pub async fn initialize_collections(&self) -> Result<MongoCollections, InfrastructureError> {
        let collections = MongoCollections {
            // HTTP logs
            http_success: self.database.collection("logs_http_success"),
            http_client_error: self.database.collection("logs_http_client_error"),
            http_server_error: self.database.collection("logs_http_server_error"),
            
            // ML logs
            ml_training: self.database.collection("logs_ml_training"),
            ml_inference: self.database.collection("logs_ml_inference"),
            ml_evaluation: self.database.collection("logs_ml_evaluation"),
            ml_experiments: self.database.collection("logs_ml_experiments"),
            
            // System logs
            system_database: self.database.collection("logs_system_database"),
            system_cache: self.database.collection("logs_system_cache"),
            system_infrastructure: self.database.collection("logs_system_infrastructure"),
            
            // Security logs
            security_auth: self.database.collection("logs_security_auth"),
            security_access: self.database.collection("logs_security_access"),
            security_audit: self.database.collection("logs_security_audit"),
        };

        // Create indexes for better query performance
        self.create_indexes(&collections).await?;

        Ok(collections)
    }

    /// Create optimal indexes for all collections
    async fn create_indexes(&self, collections: &MongoCollections) -> Result<(), InfrastructureError> {
        use mongodb::IndexModel;
        use mongodb::options::IndexOptions;

        // Common indexes for all collections
        let common_indexes = vec![
            IndexModel::builder()
                .keys(bson::doc! { "timestamp": -1 })
                .options(IndexOptions::builder().name("timestamp_desc".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(bson::doc! { "trace_id": 1 })
                .options(IndexOptions::builder().name("trace_id".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(bson::doc! { "request_id": 1 })
                .options(IndexOptions::builder().name("request_id".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(bson::doc! { "level": 1, "timestamp": -1 })
                .options(IndexOptions::builder().name("level_timestamp".to_string()).build())
                .build(),
        ];

        // HTTP specific indexes
        let http_indexes = vec![
            IndexModel::builder()
                .keys(bson::doc! { "http_context.endpoint": 1, "timestamp": -1 })
                .options(IndexOptions::builder().name("endpoint_timestamp".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(bson::doc! { "http_context.status_code": 1 })
                .options(IndexOptions::builder().name("status_code".to_string()).build())
                .build(),
        ];

        // ML specific indexes
        let ml_indexes = vec![
            IndexModel::builder()
                .keys(bson::doc! { "ml_context.experiment_id": 1, "timestamp": -1 })
                .options(IndexOptions::builder().name("experiment_timestamp".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(bson::doc! { "ml_context.model_id": 1, "timestamp": -1 })
                .options(IndexOptions::builder().name("model_timestamp".to_string()).build())
                .build(),
        ];

        // Apply indexes to HTTP collections
        for collection in [&collections.http_success, &collections.http_client_error, &collections.http_server_error] {
            let mut indexes = common_indexes.clone();
            indexes.extend(http_indexes.clone());
            collection.create_indexes(indexes, None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to create HTTP indexes: {}", e)
                ))?;
        }

        // Apply indexes to ML collections
        for collection in [&collections.ml_training, &collections.ml_inference, &collections.ml_evaluation, &collections.ml_experiments] {
            let mut indexes = common_indexes.clone();
            indexes.extend(ml_indexes.clone());
            collection.create_indexes(indexes, None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to create ML indexes: {}", e)
                ))?;
        }

        // Apply basic indexes to system and security collections
        for collection in [
            &collections.system_database, &collections.system_cache, &collections.system_infrastructure,
            &collections.security_auth, &collections.security_access, &collections.security_audit
        ] {
            collection.create_indexes(common_indexes.clone(), None).await
                .map_err(|e| InfrastructureError::DatabaseOperationError(
                    format!("Failed to create system/security indexes: {}", e)
                ))?;
        }

        Ok(())
    }

    /// Health check for MongoDB connection
    pub async fn health_check(&self) -> Result<bool, InfrastructureError> {
        match self.client
            .database("admin")
            .run_command(bson::doc! { "ping": 1 }, None)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => Err(InfrastructureError::DatabaseConnectionError(
                format!("MongoDB health check failed: {}", e)
            )),
        }
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<Document, InfrastructureError> {
        self.database
            .run_command(bson::doc! { "dbStats": 1 }, None)
            .await
            .map_err(|e| InfrastructureError::DatabaseOperationError(
                format!("Failed to get database stats: {}", e)
            ))
    }
}

/// MongoDB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBConfig {
    pub connection_string: String,
    pub database_name: String,
    pub max_pool_size: u32,
    pub min_pool_size: u32,
    pub connect_timeout_seconds: u64,
    pub server_selection_timeout_seconds: u64,
}

impl Default for MongoDBConfig {
    fn default() -> Self {
        Self {
            connection_string: "mongodb://admin:admin123@localhost:27017/".to_string(),
            database_name: "rusted_ca_logs".to_string(),
            max_pool_size: 100,
            min_pool_size: 5,
            connect_timeout_seconds: 10,
            server_selection_timeout_seconds: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mongodb_config_default() {
        let config = MongoDBConfig::default();
        assert_eq!(config.database_name, "rusted_ca_logs");
        assert_eq!(config.max_pool_size, 100);
    }

    // Note: Integration tests require MongoDB running
    // #[tokio::test]
    // async fn test_mongodb_connection() {
    //     let config = MongoDBConfig::default();
    //     let connection = MongoDBConnection::new(&config.connection_string).await;
    //     assert!(connection.is_ok());
    // }
}