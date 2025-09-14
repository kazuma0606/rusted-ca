use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::domain::entity::log_entry::{LogLevel, LogCategory};
use crate::shared::error::infrastructure_error::InfrastructureError;

/// Complete logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub mongodb: MongoDBLoggingConfig,
    pub collector: LogCollectorConfig,
    pub middleware: MiddlewareConfig,
    pub retention: RetentionConfig,
    pub performance: PerformanceConfig,
    pub security: SecurityConfig,
}

/// MongoDB-specific logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBLoggingConfig {
    pub connection_string: String,
    pub database_name: String,
    pub max_pool_size: u32,
    pub min_pool_size: u32,
    pub connect_timeout_seconds: u64,
    pub server_selection_timeout_seconds: u64,
    pub enable_ssl: bool,
    pub ssl_cert_path: Option<String>,
}

/// Log collector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCollectorConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub buffer_max_size: usize,
    pub max_retry_attempts: u32,
    pub retry_backoff_ms: u64,
    pub enable_failover_to_file: bool,
    pub failover_directory: String,
    pub worker_threads: usize,
}

/// HTTP middleware logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    pub enable_request_body_logging: bool,
    pub enable_response_body_logging: bool,
    pub enable_header_logging: bool,
    pub enable_query_param_logging: bool,
    pub max_body_size_bytes: u64,
    pub sensitive_headers: Vec<String>,
    pub sensitive_query_params: Vec<String>,
    pub excluded_endpoints: Vec<String>,
}

/// Log retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub collection_retention_days: HashMap<String, i64>,
    pub enable_auto_cleanup: bool,
    pub cleanup_interval_hours: u64,
    pub archive_before_delete: bool,
    pub archive_directory: Option<String>,
}

/// Performance and resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub log_level_filters: HashMap<LogCategory, LogLevel>,
    pub enable_sampling: bool,
    pub sample_rates: HashMap<LogCategory, f64>,
    pub enable_compression: bool,
    pub compression_algorithm: CompressionAlgorithm,
    pub max_memory_usage_mb: u64,
}

/// Security configuration for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_encryption_at_rest: bool,
    pub encryption_key_path: Option<String>,
    pub enable_audit_logging: bool,
    pub audit_log_path: String,
    pub enable_pii_redaction: bool,
    pub pii_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Zstd,
    Lz4,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            mongodb: MongoDBLoggingConfig::default(),
            collector: LogCollectorConfig::default(),
            middleware: MiddlewareConfig::default(),
            retention: RetentionConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for MongoDBLoggingConfig {
    fn default() -> Self {
        Self {
            connection_string: "mongodb://admin:admin123@localhost:27017/".to_string(),
            database_name: "rusted_ca_logs".to_string(),
            max_pool_size: 100,
            min_pool_size: 5,
            connect_timeout_seconds: 10,
            server_selection_timeout_seconds: 5,
            enable_ssl: false,
            ssl_cert_path: None,
        }
    }
}

impl Default for LogCollectorConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            flush_interval_ms: 5000,
            buffer_max_size: 10000,
            max_retry_attempts: 3,
            retry_backoff_ms: 1000,
            enable_failover_to_file: true,
            failover_directory: "./logs/failover".to_string(),
            worker_threads: 4,
        }
    }
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enable_request_body_logging: false,
            enable_response_body_logging: false,
            enable_header_logging: true,
            enable_query_param_logging: true,
            max_body_size_bytes: 1024 * 1024, // 1MB
            sensitive_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
                "x-session-token".to_string(),
            ],
            sensitive_query_params: vec![
                "password".to_string(),
                "token".to_string(),
                "key".to_string(),
                "secret".to_string(),
            ],
            excluded_endpoints: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/favicon.ico".to_string(),
            ],
        }
    }
}

impl Default for RetentionConfig {
    fn default() -> Self {
        let mut collection_retention_days = HashMap::new();
        
        // HTTP logs retention
        collection_retention_days.insert("logs_http_success".to_string(), 30);
        collection_retention_days.insert("logs_http_client_error".to_string(), 60);
        collection_retention_days.insert("logs_http_server_error".to_string(), 90);
        
        // ML logs retention (keep experiments longer)
        collection_retention_days.insert("logs_ml_training".to_string(), 90);
        collection_retention_days.insert("logs_ml_inference".to_string(), 30);
        collection_retention_days.insert("logs_ml_evaluation".to_string(), 90);
        collection_retention_days.insert("logs_ml_experiments".to_string(), 365);
        
        // System logs retention
        collection_retention_days.insert("logs_system_database".to_string(), 60);
        collection_retention_days.insert("logs_system_cache".to_string(), 30);
        collection_retention_days.insert("logs_system_infrastructure".to_string(), 60);
        
        // Security logs retention (compliance requirements)
        collection_retention_days.insert("logs_security_auth".to_string(), 180);
        collection_retention_days.insert("logs_security_access".to_string(), 180);
        collection_retention_days.insert("logs_security_audit".to_string(), 365);

        Self {
            collection_retention_days,
            enable_auto_cleanup: true,
            cleanup_interval_hours: 24, // Daily cleanup
            archive_before_delete: false,
            archive_directory: None,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        let mut log_level_filters = HashMap::new();
        log_level_filters.insert(LogCategory::Http, LogLevel::Info);
        log_level_filters.insert(LogCategory::ML, LogLevel::Info);
        log_level_filters.insert(LogCategory::Database, LogLevel::Warn);
        log_level_filters.insert(LogCategory::Cache, LogLevel::Warn);
        log_level_filters.insert(LogCategory::Auth, LogLevel::Info);
        log_level_filters.insert(LogCategory::System, LogLevel::Info);
        log_level_filters.insert(LogCategory::Security, LogLevel::Info);

        let mut sample_rates = HashMap::new();
        sample_rates.insert(LogCategory::Http, 1.0);  // Log all HTTP requests
        sample_rates.insert(LogCategory::ML, 1.0);    // Log all ML operations
        sample_rates.insert(LogCategory::Database, 0.1); // Sample 10% of DB operations
        sample_rates.insert(LogCategory::Cache, 0.1); // Sample 10% of cache operations
        sample_rates.insert(LogCategory::Auth, 1.0);  // Log all auth operations
        sample_rates.insert(LogCategory::System, 0.5); // Sample 50% of system logs
        sample_rates.insert(LogCategory::Security, 1.0); // Log all security events

        Self {
            log_level_filters,
            enable_sampling: false, // Disabled by default
            sample_rates,
            enable_compression: false,
            compression_algorithm: CompressionAlgorithm::None,
            max_memory_usage_mb: 512,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_encryption_at_rest: false,
            encryption_key_path: None,
            enable_audit_logging: false,
            audit_log_path: "./logs/audit.log".to_string(),
            enable_pii_redaction: true,
            pii_patterns: vec![
                r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b".to_string(), // Credit card
                r"\b\d{3}-\d{2}-\d{4}\b".to_string(),                      // SSN
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(), // Email (partial)
                r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b".to_string(),    // IP Address
            ],
        }
    }
}

impl LoggingConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, InfrastructureError> {
        let mut config = Self::default();

        // MongoDB configuration from environment
        if let Ok(connection_string) = std::env::var("MONGODB_CONNECTION_STRING") {
            config.mongodb.connection_string = connection_string;
        }
        if let Ok(database_name) = std::env::var("MONGODB_DATABASE_NAME") {
            config.mongodb.database_name = database_name;
        }

        // Log collector configuration
        if let Ok(batch_size) = std::env::var("LOG_BATCH_SIZE") {
            config.collector.batch_size = batch_size.parse()
                .map_err(|e| InfrastructureError::Configuration {
                    key: "LOG_BATCH_SIZE".to_string(),
                    message: format!("Invalid LOG_BATCH_SIZE: {}", e)
                })?;
        }
        if let Ok(flush_interval) = std::env::var("LOG_FLUSH_INTERVAL_MS") {
            config.collector.flush_interval_ms = flush_interval.parse()
                .map_err(|e| InfrastructureError::Configuration {
                    key: "LOG_FLUSH_INTERVAL_MS".to_string(),
                    message: format!("Invalid LOG_FLUSH_INTERVAL_MS: {}", e)
                })?;
        }

        // Middleware configuration
        if let Ok(enable_body_logging) = std::env::var("LOG_ENABLE_BODY_LOGGING") {
            config.middleware.enable_request_body_logging = enable_body_logging.parse()
                .map_err(|e| InfrastructureError::Configuration {
                    key: "LOG_ENABLE_BODY_LOGGING".to_string(),
                    message: format!("Invalid LOG_ENABLE_BODY_LOGGING: {}", e)
                })?;
        }

        // Security configuration
        if let Ok(enable_pii_redaction) = std::env::var("LOG_ENABLE_PII_REDACTION") {
            config.security.enable_pii_redaction = enable_pii_redaction.parse()
                .map_err(|e| InfrastructureError::Configuration {
                    key: "LOG_ENABLE_PII_REDACTION".to_string(),
                    message: format!("Invalid LOG_ENABLE_PII_REDACTION: {}", e)
                })?;
        }

        Ok(config)
    }

    /// Load configuration from file (TOML format)
    pub fn from_file(path: &str) -> Result<Self, InfrastructureError> {
        use std::fs;
        
        let content = fs::read_to_string(path)
            .map_err(|e| InfrastructureError::FileOperation {
                path: path.to_string(),
                message: format!("Failed to read logging config file: {}", e)
            })?;

        toml::from_str(&content)
            .map_err(|e| InfrastructureError::Configuration {
                key: "toml_parse".to_string(),
                message: format!("Failed to parse logging config: {}", e)
            })
    }

    /// Save configuration to file (TOML format)
    pub fn to_file(&self, path: &str) -> Result<(), InfrastructureError> {
        use std::fs;

        let content = toml::to_string_pretty(self)
            .map_err(|e| InfrastructureError::Configuration {
                key: "toml_serialize".to_string(),
                message: format!("Failed to serialize logging config: {}", e)
            })?;

        fs::write(path, content)
            .map_err(|e| InfrastructureError::FileOperation {
                path: path.to_string(),
                message: format!("Failed to write logging config: {}", e)
            })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), InfrastructureError> {
        // Validate MongoDB configuration
        if self.mongodb.connection_string.is_empty() {
            return Err(InfrastructureError::Configuration {
                key: "mongodb.connection_string".to_string(),
                message: "MongoDB connection string cannot be empty".to_string()
            });
        }
        if self.mongodb.max_pool_size < self.mongodb.min_pool_size {
            return Err(InfrastructureError::Configuration {
                key: "mongodb.pool_size".to_string(),
                message: "MongoDB max_pool_size must be >= min_pool_size".to_string()
            });
        }

        // Validate collector configuration
        if self.collector.batch_size == 0 {
            return Err(InfrastructureError::Configuration {
                key: "collector.batch_size".to_string(),
                message: "Log collector batch_size must be > 0".to_string()
            });
        }
        if self.collector.buffer_max_size < self.collector.batch_size {
            return Err(InfrastructureError::Configuration {
                key: "collector.buffer_max_size".to_string(),
                message: "Buffer max_size must be >= batch_size".to_string()
            });
        }

        // Validate retention configuration
        for (collection, days) in &self.retention.collection_retention_days {
            if *days <= 0 {
                return Err(InfrastructureError::Configuration {
                    key: format!("retention.{}", collection),
                    message: format!("Retention days for {} must be > 0", collection)
                });
            }
        }

        // Validate performance configuration
        for (category, rate) in &self.performance.sample_rates {
            if *rate < 0.0 || *rate > 1.0 {
                return Err(InfrastructureError::Configuration {
                    key: format!("performance.sample_rate.{:?}", category),
                    message: format!("Sample rate for {:?} must be between 0.0 and 1.0", category)
                });
            }
        }

        Ok(())
    }

    /// Get retention days for a specific collection
    pub fn get_retention_days(&self, collection_name: &str) -> i64 {
        self.retention.collection_retention_days
            .get(collection_name)
            .copied()
            .unwrap_or(30) // Default to 30 days
    }

    /// Check if logging is enabled for a specific category and level
    pub fn is_logging_enabled(&self, category: &LogCategory, level: &LogLevel) -> bool {
        if let Some(min_level) = self.performance.log_level_filters.get(category) {
            self.level_meets_threshold(level, min_level)
        } else {
            true // Default to enabled
        }
    }

    /// Check if an endpoint should be excluded from logging
    pub fn is_endpoint_excluded(&self, endpoint: &str) -> bool {
        self.middleware.excluded_endpoints.iter()
            .any(|excluded| endpoint.starts_with(excluded))
    }

    /// Get sample rate for a category
    pub fn get_sample_rate(&self, category: &LogCategory) -> f64 {
        if self.performance.enable_sampling {
            self.performance.sample_rates
                .get(category)
                .copied()
                .unwrap_or(1.0)
        } else {
            1.0 // No sampling if disabled
        }
    }

    /// Check if a log level meets the minimum threshold
    fn level_meets_threshold(&self, level: &LogLevel, min_level: &LogLevel) -> bool {
        self.level_to_number(level) >= self.level_to_number(min_level)
    }

    /// Convert log level to numeric value for comparison
    fn level_to_number(&self, level: &LogLevel) -> u8 {
        match level {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
            LogLevel::Fatal => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.mongodb.database_name, "rusted_ca_logs");
        assert_eq!(config.collector.batch_size, 100);
        assert!(config.middleware.enable_header_logging);
    }

    #[test]
    fn test_config_validation() {
        let mut config = LoggingConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid configuration
        config.collector.batch_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_retention_days() {
        let config = LoggingConfig::default();
        assert_eq!(config.get_retention_days("logs_http_success"), 30);
        assert_eq!(config.get_retention_days("logs_ml_experiments"), 365);
        assert_eq!(config.get_retention_days("nonexistent"), 30); // Default
    }

    #[test]
    fn test_logging_level_filtering() {
        let config = LoggingConfig::default();
        assert!(config.is_logging_enabled(&LogCategory::Http, &LogLevel::Info));
        assert!(config.is_logging_enabled(&LogCategory::Http, &LogLevel::Error));
        assert!(!config.is_logging_enabled(&LogCategory::Database, &LogLevel::Info));
        assert!(config.is_logging_enabled(&LogCategory::Database, &LogLevel::Error));
    }

    #[test]
    fn test_endpoint_exclusion() {
        let config = LoggingConfig::default();
        assert!(config.is_endpoint_excluded("/health"));
        assert!(config.is_endpoint_excluded("/health/status"));
        assert!(!config.is_endpoint_excluded("/api/users"));
    }
}