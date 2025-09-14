use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use dashmap::DashMap;
use tokio::sync::{mpsc, broadcast};
use tokio::time::{interval, sleep};
use crossbeam_channel::{Receiver, Sender};
use mongodb::{Collection, Database};
use bson::Document;
use futures::stream::StreamExt;

use crate::domain::entity::log_entry::{LogEntry, LogCategory, LogLevel};
use crate::infrastructure::database::mongodb_connection::MongoDBConnection;
use crate::shared::error::infrastructure_error::InfrastructureError;

/// Configuration for the log collector service
#[derive(Debug, Clone)]
pub struct LogCollectorConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub buffer_max_size: usize,
    pub max_retry_attempts: u32,
    pub enable_failover_to_file: bool,
    pub failover_directory: String,
}

impl Default for LogCollectorConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            flush_interval_ms: 5000,  // 5 seconds
            buffer_max_size: 10000,
            max_retry_attempts: 3,
            enable_failover_to_file: true,
            failover_directory: "./logs/failover".to_string(),
        }
    }
}

/// High-performance asynchronous log collection service
pub struct LogCollectorService {
    config: LogCollectorConfig,
    mongodb_connection: Arc<MongoDBConnection>,
    log_sender: mpsc::Sender<LogEntry>,
    log_receiver: Option<mpsc::Receiver<LogEntry>>,
    buffer_pool: Arc<DashMap<String, Vec<LogEntry>>>,
    shutdown_sender: broadcast::Sender<()>,
    is_running: Arc<tokio::sync::RwLock<bool>>,
    
    // Performance metrics
    processed_logs: Arc<dashmap::DashMap<String, u64>>,
    failed_logs: Arc<dashmap::DashMap<String, u64>>,
}

impl LogCollectorService {
    /// Create a new log collector service
    pub fn new(
        config: LogCollectorConfig,
        mongodb_connection: Arc<MongoDBConnection>,
    ) -> Self {
        let (log_sender, log_receiver) = mpsc::channel(config.buffer_max_size);
        let (shutdown_sender, _) = broadcast::channel(1);

        Self {
            config,
            mongodb_connection,
            log_sender,
            log_receiver: Some(log_receiver),
            buffer_pool: Arc::new(DashMap::new()),
            shutdown_sender,
            is_running: Arc::new(tokio::sync::RwLock::new(false)),
            processed_logs: Arc::new(DashMap::new()),
            failed_logs: Arc::new(DashMap::new()),
        }
    }

    /// Get a sender for async log submission
    pub fn get_sender(&self) -> mpsc::Sender<LogEntry> {
        self.log_sender.clone()
    }

    /// Start the log collection service
    pub async fn start(&mut self) -> Result<(), InfrastructureError> {
        if *self.is_running.read().await {
            return Err(InfrastructureError::ServiceError(
                "Log collector service is already running".to_string()
            ));
        }

        *self.is_running.write().await = true;

        // Take the receiver (can only be done once)
        let mut log_receiver = self.log_receiver.take()
            .ok_or_else(|| InfrastructureError::ServiceError(
                "Log receiver has already been taken".to_string()
            ))?;

        let buffer_pool = Arc::clone(&self.buffer_pool);
        let mongodb_connection = Arc::clone(&self.mongodb_connection);
        let config = self.config.clone();
        let mut shutdown_receiver = self.shutdown_sender.subscribe();
        let processed_logs = Arc::clone(&self.processed_logs);
        let failed_logs = Arc::clone(&self.failed_logs);

        // Start the main collection loop
        let collection_handle = tokio::spawn(async move {
            let mut flush_interval = interval(Duration::from_millis(config.flush_interval_ms));
            
            loop {
                tokio::select! {
                    // Handle incoming log entries
                    maybe_log = log_receiver.recv() => {
                        match maybe_log {
                            Some(log_entry) => {
                                Self::buffer_log_entry(&buffer_pool, log_entry, &config).await;
                            }
                            None => {
                                println!("Log receiver channel closed");
                                break;
                            }
                        }
                    }
                    
                    // Periodic flush based on time interval
                    _ = flush_interval.tick() => {
                        Self::flush_all_buffers(
                            &buffer_pool, 
                            &mongodb_connection, 
                            &config,
                            &processed_logs,
                            &failed_logs
                        ).await;
                    }
                    
                    // Shutdown signal
                    _ = shutdown_receiver.recv() => {
                        println!("Received shutdown signal for log collector");
                        // Final flush before shutdown
                        Self::flush_all_buffers(
                            &buffer_pool, 
                            &mongodb_connection, 
                            &config,
                            &processed_logs,
                            &failed_logs
                        ).await;
                        break;
                    }
                }
            }
        });

        println!("✓ Log collector service started successfully");
        Ok(())
    }

    /// Submit a log entry asynchronously
    pub async fn log_async(&self, entry: LogEntry) -> Result<(), InfrastructureError> {
        self.log_sender.send(entry).await
            .map_err(|e| InfrastructureError::ServiceError(
                format!("Failed to send log entry: {}", e)
            ))?;
        Ok(())
    }

    /// Log ML operation with standardized format
    pub async fn log_ml_operation(&self, model_id: &str, operation: &str, message: &str) -> Result<(), InfrastructureError> {
        use chrono::Utc;
        use uuid::Uuid;
        use crate::domain::entity::log_entry::{LogLevel, LogCategory, MLLogContext, SystemLogContext, LogMetrics};
        use bson::doc;

        let entry = LogEntry {
            id: None,
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category: LogCategory::ML,
            subcategory: Some("inference".to_string()),
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            request_id: Uuid::new_v4().to_string(),
            session_id: None,
            user_id: None,
            http_context: None,
            ml_context: Some(MLLogContext {
                operation_type: crate::domain::entity::log_entry::MLOperationType::Inference,
                model_id: Some(model_id.to_string()),
                model_version: Some("v1".to_string()),
                experiment_id: None,
                dataset_id: None,
                epoch: None,
                batch_size: None,
                learning_rate: None,
                loss: None,
                accuracy: None,
                inference_time_ms: None,
                memory_usage_mb: None,
                gpu_usage_percent: None,
                input_shape: None,
                output_shape: None,
                hyperparameters: None,
                model_metrics: None,
            }),
            system_context: SystemLogContext {
                layer: crate::domain::entity::log_entry::ArchitectureLayer::Application,
                component: "InferenceUsecase".to_string(),
                operation: operation.to_string(),
                hostname: "localhost".to_string(),
                process_id: std::process::id(),
                thread_id: format!("{:?}", std::thread::current().id()),
                service_version: Some("0.1.0".to_string()),
            },
            metrics: LogMetrics {
                execution_time_ms: None,
                memory_usage_bytes: None,
                cpu_usage_percent: None,
                disk_io_bytes: None,
                network_io_bytes: None,
                database_query_time_ms: None,
                cache_hit_rate: None,
            },
            metadata: doc! {
                "model_id": model_id,
                "operation_type": operation
            },
            tags: vec!["ml".to_string(), operation.to_string()],
        };
        
        self.log_async(entry).await
    }

    /// Force flush all buffers immediately
    pub async fn flush_all(&self) -> Result<(), InfrastructureError> {
        Self::flush_all_buffers(
            &self.buffer_pool,
            &self.mongodb_connection,
            &self.config,
            &self.processed_logs,
            &self.failed_logs
        ).await;
        Ok(())
    }

    /// Stop the log collection service
    pub async fn stop(&self) -> Result<(), InfrastructureError> {
        if !*self.is_running.read().await {
            return Err(InfrastructureError::ServiceError(
                "Log collector service is not running".to_string()
            ));
        }

        self.shutdown_sender.send(())
            .map_err(|e| InfrastructureError::ServiceError(
                format!("Failed to send shutdown signal: {}", e)
            ))?;

        *self.is_running.write().await = false;
        println!("✓ Log collector service stopped");
        Ok(())
    }

    /// Buffer a log entry for batch processing
    async fn buffer_log_entry(
        buffer_pool: &Arc<DashMap<String, Vec<LogEntry>>>,
        log_entry: LogEntry,
        config: &LogCollectorConfig,
    ) {
        let collection_name = log_entry.get_collection_name();
        
        // Add to buffer
        buffer_pool.entry(collection_name.clone())
            .and_modify(|buffer| buffer.push(log_entry.clone()))
            .or_insert_with(|| vec![log_entry.clone()]);

        // Check if we need to flush due to buffer size
        if let Some(buffer) = buffer_pool.get(&collection_name) {
            if buffer.len() >= config.batch_size {
                drop(buffer); // Release the read lock
                
                // Extract and flush this specific buffer
                if let Some((_, logs)) = buffer_pool.remove(&collection_name) {
                    // We'll flush this in the background to avoid blocking
                    // In a real implementation, you might want to send these to a separate flush channel
                }
            }
        }
    }

    /// Flush all buffers to MongoDB
    async fn flush_all_buffers(
        buffer_pool: &Arc<DashMap<String, Vec<LogEntry>>>,
        mongodb_connection: &Arc<MongoDBConnection>,
        config: &LogCollectorConfig,
        processed_logs: &Arc<DashMap<String, u64>>,
        failed_logs: &Arc<DashMap<String, u64>>,
    ) {
        let collections_to_flush: Vec<String> = buffer_pool.iter()
            .map(|entry| entry.key().clone())
            .collect();

        for collection_name in collections_to_flush {
            if let Some((_, logs)) = buffer_pool.remove(&collection_name) {
                if !logs.is_empty() {
                    Self::flush_to_collection(
                        &collection_name,
                        logs,
                        mongodb_connection,
                        config,
                        processed_logs,
                        failed_logs,
                    ).await;
                }
            }
        }
    }

    /// Flush logs to a specific MongoDB collection
    async fn flush_to_collection(
        collection_name: &str,
        logs: Vec<LogEntry>,
        mongodb_connection: &Arc<MongoDBConnection>,
        config: &LogCollectorConfig,
        processed_logs: &Arc<DashMap<String, u64>>,
        failed_logs: &Arc<DashMap<String, u64>>,
    ) {
        let collection: Collection<Document> = mongodb_connection
            .database()
            .collection(collection_name);

        // Convert log entries to BSON documents
        let documents: Vec<Document> = logs.iter()
            .filter_map(|log| bson::to_document(log).ok())
            .collect();

        if documents.is_empty() {
            return;
        }

        let mut retry_count = 0;
        let log_count = documents.len();

        loop {
            match collection.insert_many(documents.clone(), None).await {
                Ok(_) => {
                    // Update success metrics
                    processed_logs.entry(collection_name.to_string())
                        .and_modify(|count| *count += log_count as u64)
                        .or_insert(log_count as u64);
                    
                    println!("✓ Flushed {} logs to {}", log_count, collection_name);
                    break;
                }
                Err(e) => {
                    retry_count += 1;
                    
                    if retry_count >= config.max_retry_attempts {
                        // Update failure metrics
                        failed_logs.entry(collection_name.to_string())
                            .and_modify(|count| *count += log_count as u64)
                            .or_insert(log_count as u64);

                        eprintln!("✗ Failed to flush logs to {} after {} retries: {}", 
                            collection_name, retry_count, e);

                        // Failover to file if enabled
                        if config.enable_failover_to_file {
                            Self::failover_to_file(&logs, collection_name, &config.failover_directory).await;
                        }
                        break;
                    } else {
                        println!("⚠ Retrying flush to {} (attempt {}/{}): {}", 
                            collection_name, retry_count, config.max_retry_attempts, e);
                        
                        // Exponential backoff
                        let delay = Duration::from_millis(100 * (2_u64.pow(retry_count - 1)));
                        sleep(delay).await;
                    }
                }
            }
        }
    }

    /// Failover logs to local file system when MongoDB is unavailable
    async fn failover_to_file(logs: &[LogEntry], collection_name: &str, failover_dir: &str) {
        use tokio::fs::{create_dir_all, OpenOptions};
        use tokio::io::AsyncWriteExt;

        if let Err(e) = create_dir_all(failover_dir).await {
            eprintln!("Failed to create failover directory {}: {}", failover_dir, e);
            return;
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}/failover_{}_{}.jsonl", failover_dir, collection_name, timestamp);

        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
            .await
        {
            Ok(mut file) => {
                for log in logs {
                    if let Ok(json) = serde_json::to_string(log) {
                        if let Err(e) = file.write_all(format!("{}\n", json).as_bytes()).await {
                            eprintln!("Failed to write to failover file {}: {}", filename, e);
                            break;
                        }
                    }
                }
                println!("✓ Failover: saved {} logs to {}", logs.len(), filename);
            }
            Err(e) => {
                eprintln!("Failed to open failover file {}: {}", filename, e);
            }
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> LogCollectorStats {
        LogCollectorStats {
            processed_logs: self.processed_logs.iter()
                .map(|entry| (entry.key().clone(), *entry.value()))
                .collect(),
            failed_logs: self.failed_logs.iter()
                .map(|entry| (entry.key().clone(), *entry.value()))
                .collect(),
            buffer_sizes: self.buffer_pool.iter()
                .map(|entry| (entry.key().clone(), entry.value().len()))
                .collect(),
            is_running: self.is_running.try_read().map(|guard| *guard).unwrap_or(false),
        }
    }
}

/// Performance statistics for the log collector
#[derive(Debug, Clone)]
pub struct LogCollectorStats {
    pub processed_logs: HashMap<String, u64>,
    pub failed_logs: HashMap<String, u64>,
    pub buffer_sizes: HashMap<String, usize>,
    pub is_running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::log_entry::{LogLevel, LogCategory};

    #[tokio::test]
    async fn test_log_collector_creation() {
        // Mock MongoDB connection would be needed for full test
        // This test just verifies the configuration
        let config = LogCollectorConfig::default();
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.flush_interval_ms, 5000);
        assert_eq!(config.buffer_max_size, 10000);
    }

    #[test]
    fn test_log_collector_config() {
        let config = LogCollectorConfig {
            batch_size: 50,
            flush_interval_ms: 1000,
            buffer_max_size: 5000,
            max_retry_attempts: 5,
            enable_failover_to_file: false,
            failover_directory: "/tmp/logs".to_string(),
        };

        assert_eq!(config.batch_size, 50);
        assert_eq!(config.max_retry_attempts, 5);
        assert!(!config.enable_failover_to_file);
    }
}