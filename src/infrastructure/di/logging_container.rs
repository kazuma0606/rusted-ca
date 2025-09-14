use std::sync::Arc;
use tokio::sync::Mutex;

use crate::infrastructure::database::mongodb_connection::MongoDBConnection;
use crate::infrastructure::database::mongodb_initializer::MongoDBInitializer;
use crate::infrastructure::logging::log_collector_service::LogCollectorService;
use crate::infrastructure::repository::mongodb::http_log_repository::HttpLogRepository;
use crate::infrastructure::config::logging_config::LoggingConfig;
use crate::shared::middleware::logging_middleware::LoggingMiddleware;
use crate::shared::error::infrastructure_error::InfrastructureError;

/// Dependency injection container for logging services
pub struct LoggingContainer {
    pub config: LoggingConfig,
    pub mongodb_connection: Arc<MongoDBConnection>,
    pub log_collector_service: Arc<Mutex<LogCollectorService>>,
    pub http_log_repository: Arc<HttpLogRepository>,
    pub logging_middleware: LoggingMiddleware,
    pub mongodb_initializer: MongoDBInitializer,
}

impl LoggingContainer {
    /// Create a new logging container with all dependencies
    pub async fn new(config: LoggingConfig) -> Result<Self, InfrastructureError> {
        // Validate configuration first
        config.validate()?;

        // Create MongoDB connection
        let mongodb_connection = Arc::new(
            MongoDBConnection::new(&config.mongodb.connection_string).await?
        );

        // Create MongoDB initializer
        let mongodb_initializer = MongoDBInitializer::new(mongodb_connection.database().clone());

        // Initialize MongoDB collections and indexes
        mongodb_initializer.initialize_all_collections().await?;

        // Create log collector service
        let collector_config = crate::infrastructure::logging::log_collector_service::LogCollectorConfig {
            batch_size: config.collector.batch_size,
            flush_interval_ms: config.collector.flush_interval_ms,
            buffer_max_size: config.collector.buffer_max_size,
            max_retry_attempts: config.collector.max_retry_attempts,
            enable_failover_to_file: config.collector.enable_failover_to_file,
            failover_directory: config.collector.failover_directory.clone(),
        };

        let mut log_collector_service = LogCollectorService::new(
            collector_config,
            Arc::clone(&mongodb_connection),
        );

        // Start the log collector service
        log_collector_service.start().await?;
        let log_collector_service = Arc::new(Mutex::new(log_collector_service));

        // Create repositories
        let http_log_repository = Arc::new(
            HttpLogRepository::new(Arc::clone(&mongodb_connection))
        );

        // Create logging middleware
        let logging_middleware = LoggingMiddleware::with_config(
            &*log_collector_service.lock().await,
            "rusted-ca".to_string(),
            config.middleware.enable_request_body_logging,
            config.middleware.enable_response_body_logging,
        );

        println!("✓ Logging container initialized successfully");

        Ok(Self {
            config,
            mongodb_connection,
            log_collector_service,
            http_log_repository,
            logging_middleware,
            mongodb_initializer,
        })
    }

    /// Create logging container from environment variables
    pub async fn from_env() -> Result<Self, InfrastructureError> {
        let config = LoggingConfig::from_env()?;
        Self::new(config).await
    }

    /// Create logging container from configuration file
    pub async fn from_file(config_path: &str) -> Result<Self, InfrastructureError> {
        let config = LoggingConfig::from_file(config_path)?;
        Self::new(config).await
    }

    /// Get the logging middleware for Axum integration
    pub fn get_logging_middleware(&self) -> LoggingMiddleware {
        self.logging_middleware.clone()
    }

    /// Get the log collector service
    pub fn get_log_collector(&self) -> Arc<Mutex<LogCollectorService>> {
        Arc::clone(&self.log_collector_service)
    }

    /// Get HTTP log repository
    pub fn get_http_log_repository(&self) -> Arc<HttpLogRepository> {
        Arc::clone(&self.http_log_repository)
    }

    /// Get MongoDB connection
    pub fn get_mongodb_connection(&self) -> Arc<MongoDBConnection> {
        Arc::clone(&self.mongodb_connection)
    }

    /// Perform health check on all logging services
    pub async fn health_check(&self) -> Result<LoggingHealthStatus, InfrastructureError> {
        // Check MongoDB connection
        let mongodb_healthy = self.mongodb_connection.health_check().await.is_ok();

        // Check log collector service status
        let collector_stats = self.log_collector_service.lock().await.get_stats();
        let collector_healthy = collector_stats.is_running;

        // Check buffer levels
        let buffer_usage = collector_stats.buffer_sizes.values().sum::<usize>();
        let buffer_healthy = buffer_usage < self.config.collector.buffer_max_size;

        let overall_healthy = mongodb_healthy && collector_healthy && buffer_healthy;

        Ok(LoggingHealthStatus {
            overall_healthy,
            mongodb_healthy,
            collector_healthy,
            buffer_healthy,
            buffer_usage,
            processed_logs_total: collector_stats.processed_logs.values().sum(),
            failed_logs_total: collector_stats.failed_logs.values().sum(),
        })
    }

    /// Get logging statistics
    pub async fn get_statistics(&self) -> LoggingStatistics {
        let collector_stats = self.log_collector_service.lock().await.get_stats();
        
        LoggingStatistics {
            collector_running: collector_stats.is_running,
            total_processed: collector_stats.processed_logs.values().sum(),
            total_failed: collector_stats.failed_logs.values().sum(),
            buffer_sizes: collector_stats.buffer_sizes,
            processed_by_collection: collector_stats.processed_logs,
            failed_by_collection: collector_stats.failed_logs,
        }
    }

    /// Flush all pending logs immediately
    pub async fn flush_all_logs(&self) -> Result<(), InfrastructureError> {
        self.log_collector_service.lock().await.flush_all().await
    }

    /// Gracefully shutdown all logging services
    pub async fn shutdown(&self) -> Result<(), InfrastructureError> {
        println!("Shutting down logging container...");

        // Flush any remaining logs
        self.flush_all_logs().await?;

        // Stop log collector service
        self.log_collector_service.lock().await.stop().await?;

        println!("✓ Logging container shut down successfully");
        Ok(())
    }

    /// Perform maintenance tasks (cleanup old logs, optimize indexes)
    pub async fn perform_maintenance(&self) -> Result<MaintenanceReport, InfrastructureError> {
        let mut report = MaintenanceReport::default();

        if self.config.retention.enable_auto_cleanup {
            // Clean up old logs based on retention policy
            for (collection_name, retention_days) in &self.config.retention.collection_retention_days {
                match collection_name.as_str() {
                    name if name.starts_with("logs_http_") => {
                        let deleted = self.http_log_repository.cleanup_old_logs(*retention_days).await?;
                        report.deleted_logs += deleted;
                        report.cleaned_collections.push(collection_name.clone());
                    }
                    // Add other collection types when implemented
                    _ => {}
                }
            }
        }

        // Get database statistics
        if let Ok(db_stats) = self.mongodb_connection.get_stats().await {
            if let Some(size) = db_stats.get_f64("dataSize").ok() {
                report.database_size_mb = (size / 1024.0 / 1024.0) as u64;
            }
        }

        report.maintenance_completed_at = chrono::Utc::now();
        
        println!("✓ Maintenance completed: deleted {} old logs from {} collections", 
            report.deleted_logs, report.cleaned_collections.len());

        Ok(report)
    }
}

/// Health status of the logging system
#[derive(Debug, Clone)]
pub struct LoggingHealthStatus {
    pub overall_healthy: bool,
    pub mongodb_healthy: bool,
    pub collector_healthy: bool,
    pub buffer_healthy: bool,
    pub buffer_usage: usize,
    pub processed_logs_total: u64,
    pub failed_logs_total: u64,
}

/// Logging system statistics
#[derive(Debug, Clone)]
pub struct LoggingStatistics {
    pub collector_running: bool,
    pub total_processed: u64,
    pub total_failed: u64,
    pub buffer_sizes: std::collections::HashMap<String, usize>,
    pub processed_by_collection: std::collections::HashMap<String, u64>,
    pub failed_by_collection: std::collections::HashMap<String, u64>,
}

/// Maintenance operation report
#[derive(Debug, Clone, Default)]
pub struct MaintenanceReport {
    pub deleted_logs: u64,
    pub cleaned_collections: Vec<String>,
    pub database_size_mb: u64,
    pub maintenance_completed_at: chrono::DateTime<chrono::Utc>,
}

impl Drop for LoggingContainer {
    fn drop(&mut self) {
        // Note: In a real application, you might want to handle shutdown more gracefully
        // This is just a safety net
        println!("LoggingContainer is being dropped");
    }
}

/// Builder for creating LoggingContainer with custom configuration
pub struct LoggingContainerBuilder {
    config: LoggingConfig,
}

impl LoggingContainerBuilder {
    pub fn new() -> Self {
        Self {
            config: LoggingConfig::default(),
        }
    }

    pub fn with_mongodb_connection(mut self, connection_string: String) -> Self {
        self.config.mongodb.connection_string = connection_string;
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.config.collector.batch_size = batch_size;
        self
    }

    pub fn with_flush_interval(mut self, flush_interval_ms: u64) -> Self {
        self.config.collector.flush_interval_ms = flush_interval_ms;
        self
    }

    pub fn enable_request_body_logging(mut self, enable: bool) -> Self {
        self.config.middleware.enable_request_body_logging = enable;
        self
    }

    pub fn enable_response_body_logging(mut self, enable: bool) -> Self {
        self.config.middleware.enable_response_body_logging = enable;
        self
    }

    pub fn with_retention_days(mut self, collection: String, days: i64) -> Self {
        self.config.retention.collection_retention_days.insert(collection, days);
        self
    }

    pub async fn build(self) -> Result<LoggingContainer, InfrastructureError> {
        LoggingContainer::new(self.config).await
    }
}

impl Default for LoggingContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logging_container_builder() {
        let builder = LoggingContainerBuilder::new()
            .with_batch_size(50)
            .with_flush_interval(1000)
            .enable_request_body_logging(true);

        assert_eq!(builder.config.collector.batch_size, 50);
        assert_eq!(builder.config.collector.flush_interval_ms, 1000);
        assert!(builder.config.middleware.enable_request_body_logging);

        // Note: Building would require actual MongoDB connection
        // let container = builder.build().await;
        // assert!(container.is_ok());
    }

    #[test]
    fn test_health_status() {
        let health = LoggingHealthStatus {
            overall_healthy: true,
            mongodb_healthy: true,
            collector_healthy: true,
            buffer_healthy: true,
            buffer_usage: 100,
            processed_logs_total: 1000,
            failed_logs_total: 5,
        };

        assert!(health.overall_healthy);
        assert_eq!(health.processed_logs_total, 1000);
    }
}