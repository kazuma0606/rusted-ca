use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use bson::{Document, oid::ObjectId};
use uuid::Uuid;

/// Main log entry entity for MongoDB storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    
    // Core Fields
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub category: LogCategory,
    pub subcategory: Option<String>,
    pub message: String,
    
    // Request Tracing
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub request_id: String,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    
    // Context-specific data
    pub http_context: Option<HttpLogContext>,
    pub ml_context: Option<MLLogContext>,
    pub system_context: SystemLogContext,
    
    // Performance Metrics
    pub metrics: LogMetrics,
    
    // Structured Data
    pub metadata: Document,
    pub tags: Vec<String>,
}

/// HTTP-specific logging context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpLogContext {
    pub method: String,
    pub endpoint: String,
    pub status_code: u16,
    pub user_agent: Option<String>,
    pub ip_address: String,
    pub content_length: Option<u64>,
    pub response_time_ms: u64,
    pub query_params: Option<Document>,
    pub request_headers: Option<Document>,
    pub response_headers: Option<Document>,
}

/// Machine Learning specific logging context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLLogContext {
    pub operation_type: MLOperationType,
    pub model_id: Option<String>,
    pub model_version: Option<String>,
    pub experiment_id: Option<String>,
    pub dataset_id: Option<String>,
    
    // Training specific
    pub epoch: Option<u32>,
    pub batch_size: Option<u32>,
    pub learning_rate: Option<f64>,
    
    // Performance metrics
    pub loss: Option<f64>,
    pub accuracy: Option<f64>,
    pub inference_time_ms: Option<u64>,
    
    // Resource usage
    pub memory_usage_mb: Option<u64>,
    pub gpu_usage_percent: Option<f32>,
    
    // Data shape info
    pub input_shape: Option<Vec<usize>>,
    pub output_shape: Option<Vec<usize>>,
    
    // Additional ML metadata
    pub hyperparameters: Option<Document>,
    pub model_metrics: Option<Document>,
}

/// System-level logging context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLogContext {
    pub layer: ArchitectureLayer,
    pub component: String,
    pub operation: String,
    pub hostname: String,
    pub process_id: u32,
    pub thread_id: String,
    pub service_version: Option<String>,
}

/// Performance and resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetrics {
    pub execution_time_ms: Option<u64>,
    pub memory_usage_bytes: Option<u64>,
    pub cpu_usage_percent: Option<f32>,
    pub disk_io_bytes: Option<u64>,
    pub network_io_bytes: Option<u64>,
    pub database_query_time_ms: Option<u64>,
    pub cache_hit_rate: Option<f32>,
}

/// Log severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// Main log categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LogCategory {
    Http,
    ML,
    Database,
    Cache,
    Auth,
    System,
    Security,
}

/// ML operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MLOperationType {
    Training,
    Inference,
    Evaluation,
    DataPreprocessing,
    ModelSaving,
    ModelLoading,
    ExperimentTracking,
    Hyperparameters,
    DatasetValidation,
}

/// Architecture layer identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitectureLayer {
    Presentation,
    Application,
    Domain,
    Infrastructure,
}

impl LogEntry {
    /// Create a new log entry with basic information
    pub fn new(
        level: LogLevel,
        category: LogCategory,
        message: String,
        trace_id: String,
    ) -> Self {
        let now = Utc::now();
        let request_id = Uuid::new_v4().to_string();
        let span_id = Uuid::new_v4().to_string();

        Self {
            id: None,
            timestamp: now,
            level,
            category,
            subcategory: None,
            message,
            trace_id,
            span_id,
            parent_span_id: None,
            request_id,
            session_id: None,
            user_id: None,
            http_context: None,
            ml_context: None,
            system_context: SystemLogContext::default(),
            metrics: LogMetrics::default(),
            metadata: Document::new(),
            tags: Vec::new(),
        }
    }

    /// Create HTTP-specific log entry
    pub fn http(
        level: LogLevel,
        message: String,
        trace_id: String,
        http_context: HttpLogContext,
    ) -> Self {
        let mut entry = Self::new(level, LogCategory::Http, message, trace_id);
        entry.http_context = Some(http_context);
        entry
    }

    /// Create ML-specific log entry
    pub fn ml(
        level: LogLevel,
        message: String,
        trace_id: String,
        ml_context: MLLogContext,
    ) -> Self {
        let mut entry = Self::new(level, LogCategory::ML, message, trace_id);
        entry.ml_context = Some(ml_context);
        entry
    }

    /// Add metadata to log entry
    pub fn with_metadata(mut self, key: &str, value: bson::Bson) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Add tags to log entry
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Add user context
    pub fn with_user(mut self, user_id: String, session_id: Option<String>) -> Self {
        self.user_id = Some(user_id);
        self.session_id = session_id;
        self
    }

    /// Add parent span for distributed tracing
    pub fn with_parent_span(mut self, parent_span_id: String) -> Self {
        self.parent_span_id = Some(parent_span_id);
        self
    }

    /// Update metrics
    pub fn with_metrics(mut self, metrics: LogMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set subcategory for more specific classification
    pub fn with_subcategory(mut self, subcategory: String) -> Self {
        self.subcategory = Some(subcategory);
        self
    }

    /// Check if this is an error level log
    pub fn is_error(&self) -> bool {
        matches!(self.level, LogLevel::Error | LogLevel::Fatal)
    }

    /// Check if this is a success level log (Info with successful HTTP status)
    pub fn is_success(&self) -> bool {
        matches!(self.level, LogLevel::Info) &&
        self.http_context.as_ref()
            .map(|ctx| (200..300).contains(&ctx.status_code))
            .unwrap_or(false)
    }

    /// Get collection name based on category and level
    pub fn get_collection_name(&self) -> String {
        match (&self.category, &self.level) {
            (LogCategory::Http, LogLevel::Info) if self.is_success() => "logs_http_success".to_string(),
            (LogCategory::Http, LogLevel::Error | LogLevel::Fatal) => "logs_http_server_error".to_string(),
            (LogCategory::Http, LogLevel::Warn) => "logs_http_client_error".to_string(),
            (LogCategory::ML, _) => match self.ml_context.as_ref() {
                Some(ctx) => match ctx.operation_type {
                    MLOperationType::Training => "logs_ml_training".to_string(),
                    MLOperationType::Inference => "logs_ml_inference".to_string(),
                    MLOperationType::Evaluation => "logs_ml_evaluation".to_string(),
                    MLOperationType::ExperimentTracking => "logs_ml_experiments".to_string(),
                    _ => "logs_ml_training".to_string(),
                },
                None => "logs_ml_training".to_string(),
            },
            (LogCategory::Database, _) => "logs_system_database".to_string(),
            (LogCategory::Cache, _) => "logs_system_cache".to_string(),
            (LogCategory::System, _) => "logs_system_infrastructure".to_string(),
            (LogCategory::Auth, _) => "logs_security_auth".to_string(),
            (LogCategory::Security, _) => "logs_security_access".to_string(),
            _ => "logs_system_infrastructure".to_string(),
        }
    }
}

impl Default for SystemLogContext {
    fn default() -> Self {
        Self {
            layer: ArchitectureLayer::Infrastructure,
            component: "unknown".to_string(),
            operation: "unknown".to_string(),
            hostname: gethostname::gethostname().to_string_lossy().to_string(),
            process_id: std::process::id(),
            thread_id: format!("{:?}", std::thread::current().id()),
            service_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }
}

impl Default for LogMetrics {
    fn default() -> Self {
        Self {
            execution_time_ms: None,
            memory_usage_bytes: None,
            cpu_usage_percent: None,
            disk_io_bytes: None,
            network_io_bytes: None,
            database_query_time_ms: None,
            cache_hit_rate: None,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

impl std::fmt::Display for LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogCategory::Http => write!(f, "HTTP"),
            LogCategory::ML => write!(f, "ML"),
            LogCategory::Database => write!(f, "DATABASE"),
            LogCategory::Cache => write!(f, "CACHE"),
            LogCategory::Auth => write!(f, "AUTH"),
            LogCategory::System => write!(f, "SYSTEM"),
            LogCategory::Security => write!(f, "SECURITY"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let trace_id = "test-trace-123".to_string();
        let entry = LogEntry::new(
            LogLevel::Info,
            LogCategory::Http,
            "Test message".to_string(),
            trace_id.clone(),
        );

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.category, LogCategory::Http);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.trace_id, trace_id);
        assert!(entry.id.is_none());
    }

    #[test]
    fn test_collection_name_generation() {
        let trace_id = "test-trace-123".to_string();
        
        // HTTP success
        let http_entry = LogEntry::http(
            LogLevel::Info,
            "Success".to_string(),
            trace_id.clone(),
            HttpLogContext {
                method: "GET".to_string(),
                endpoint: "/api/test".to_string(),
                status_code: 200,
                user_agent: None,
                ip_address: "127.0.0.1".to_string(),
                content_length: None,
                response_time_ms: 100,
                query_params: None,
                request_headers: None,
                response_headers: None,
            }
        );
        assert_eq!(http_entry.get_collection_name(), "logs_http_success");

        // ML training
        let ml_entry = LogEntry::ml(
            LogLevel::Info,
            "Training".to_string(),
            trace_id,
            MLLogContext {
                operation_type: MLOperationType::Training,
                model_id: Some("model-123".to_string()),
                model_version: None,
                experiment_id: None,
                dataset_id: None,
                epoch: Some(1),
                batch_size: Some(32),
                learning_rate: Some(0.001),
                loss: Some(0.5),
                accuracy: Some(0.85),
                inference_time_ms: None,
                memory_usage_mb: None,
                gpu_usage_percent: None,
                input_shape: None,
                output_shape: None,
                hyperparameters: None,
                model_metrics: None,
            }
        );
        assert_eq!(ml_entry.get_collection_name(), "logs_ml_training");
    }

    #[test]
    fn test_log_entry_builder_pattern() {
        let entry = LogEntry::new(
            LogLevel::Info,
            LogCategory::System,
            "Test".to_string(),
            "trace-123".to_string(),
        )
        .with_metadata("key", bson::Bson::String("value".to_string()))
        .with_tags(vec!["tag1".to_string(), "tag2".to_string()])
        .with_user("user-123".to_string(), Some("session-456".to_string()));

        assert_eq!(entry.user_id, Some("user-123".to_string()));
        assert_eq!(entry.session_id, Some("session-456".to_string()));
        assert_eq!(entry.tags.len(), 2);
        assert!(entry.metadata.contains_key("key"));
    }
}