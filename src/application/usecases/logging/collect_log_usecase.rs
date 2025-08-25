use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::collect_log_request::CollectLogRequest;
use crate::domain::{
    entity::log_entry::LogEntry,
    repository::log_repository::LogRepositoryInterface,
    value_object::log_id::LogId,
};
use crate::shared::error::application_error::ApplicationError;
use crate::shared::metrics::collector::MetricsCollectorInterface;

pub type ApplicationResult<T> = Result<T, ApplicationError>;

pub trait IdGeneratorInterface: Send + Sync {
    fn generate_log_id(&self) -> LogId;
}

pub struct UuidGenerator;

impl UuidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGeneratorInterface for UuidGenerator {
    fn generate_log_id(&self) -> LogId {
        LogId::from(Uuid::new_v4())
    }
}

pub struct CollectLogUsecase {
    log_repository: Arc<dyn LogRepositoryInterface>,
    metrics_collector: Arc<dyn MetricsCollectorInterface>,
    id_generator: Arc<dyn IdGeneratorInterface>,
}

impl CollectLogUsecase {
    pub fn new(
        log_repository: Arc<dyn LogRepositoryInterface>,
        metrics_collector: Arc<dyn MetricsCollectorInterface>,
        id_generator: Arc<dyn IdGeneratorInterface>,
    ) -> Self {
        Self {
            log_repository,
            metrics_collector,
            id_generator,
        }
    }

    pub async fn collect(&self, request: CollectLogRequest) -> ApplicationResult<LogId> {
        // Generate unique log ID
        let log_id = self.id_generator.generate_log_id();

        // Create domain entity
        let log_entry = LogEntry::new(
            log_id.clone(),
            request.timestamp,
            request.level,
            request.message,
            request.request_id,
            request.user_context,
            request.http_context,
            request.architecture_layer,
            request.operation,
            request.metadata,
        )
        .map_err(|e| ApplicationError::ValidationError(e.to_string()))?;

        // Validate business rules
        self.validate_log_entry(&log_entry)?;

        // Store log entry
        self.log_repository
            .store(&log_entry)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        // Record metrics
        if let Err(e) = self.metrics_collector.record_log_event(&log_entry) {
            // Log metrics collection failure but don't fail the entire operation
            eprintln!("Failed to record log metrics: {:?}", e);
        }

        // Send critical alerts if needed
        if log_entry.level() == crate::domain::value_object::log_level::LogLevel::Critical {
            self.handle_critical_alert(&log_entry).await;
        }

        Ok(log_id)
    }

    fn validate_log_entry(&self, entry: &LogEntry) -> ApplicationResult<()> {
        // Additional application-level validation
        if entry.message().trim().is_empty() {
            return Err(ApplicationError::ValidationError(
                "Log message cannot be empty".to_string(),
            ));
        }

        // Check for potentially sensitive information
        if self.contains_sensitive_info(entry.message()) {
            return Err(ApplicationError::ValidationError(
                "Log message contains potentially sensitive information".to_string(),
            ));
        }

        Ok(())
    }

    fn contains_sensitive_info(&self, message: &str) -> bool {
        let sensitive_patterns = ["password", "token", "secret", "key", "api_key"];
        let message_lower = message.to_lowercase();
        
        sensitive_patterns
            .iter()
            .any(|pattern| message_lower.contains(pattern))
    }

    async fn handle_critical_alert(&self, entry: &LogEntry) {
        // This would integrate with Discord notification system
        // For now, just log to stderr
        eprintln!(
            "🚨 CRITICAL ALERT: {} - Request ID: {} - User: {:?}",
            entry.message(),
            entry.request_id(),
            entry.user_context()
        );
        
        // In a real implementation, this would:
        // 1. Send Discord notification
        // 2. Create incident ticket
        // 3. Page on-call engineer
        // 4. Update monitoring dashboards
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_object::{
        architecture_layer::ArchitectureLayer, http_context::HttpContext, log_level::LogLevel,
        log_metadata::LogMetadata, operation::Operation, request_id::RequestId,
    };
    use crate::domain::repository::log_repository::LogRepositoryInterface;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Mutex;

    // Mock implementations for testing
    struct MockLogRepository {
        stored_logs: Mutex<Vec<LogEntry>>,
    }

    impl MockLogRepository {
        fn new() -> Self {
            Self {
                stored_logs: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl LogRepositoryInterface for MockLogRepository {
        async fn store(&self, entry: &LogEntry) -> Result<(), crate::shared::error::domain_error::DomainError> {
            self.stored_logs.lock().unwrap().push(entry.clone());
            Ok(())
        }

        async fn find_by_id(&self, _id: &LogId) -> Result<Option<LogEntry>, crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn find_by_criteria(&self, _criteria: &crate::domain::repository::log_repository::LogSearchCriteria) -> Result<crate::domain::repository::log_repository::LogSearchResult, crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn update_analysis_status(&self, _id: &LogId, _status: crate::domain::value_object::analysis_status::AnalysisStatus) -> Result<(), crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn add_tags(&self, _id: &LogId, _tags: Vec<String>) -> Result<(), crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn remove_tag(&self, _id: &LogId, _tag: &str) -> Result<(), crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn add_related_log(&self, _id: &LogId, _related_id: LogId) -> Result<(), crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn count_by_criteria(&self, _criteria: &crate::domain::repository::log_repository::LogSearchCriteria) -> Result<usize, crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn delete_older_than(&self, _cutoff_time: chrono::DateTime<chrono::Utc>) -> Result<usize, crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn find_by_request_id(&self, _request_id: &RequestId) -> Result<Vec<LogEntry>, crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn find_errors_in_range(&self, _start: chrono::DateTime<chrono::Utc>, _end: chrono::DateTime<chrono::Utc>) -> Result<Vec<LogEntry>, crate::shared::error::domain_error::DomainError> {
            todo!()
        }

        async fn get_performance_metrics(&self, _start: chrono::DateTime<chrono::Utc>, _end: chrono::DateTime<chrono::Utc>) -> Result<crate::domain::repository::log_repository::PerformanceMetrics, crate::shared::error::domain_error::DomainError> {
            todo!()
        }
    }

    struct MockMetricsCollector;

    impl MockMetricsCollector {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl MetricsCollectorInterface for MockMetricsCollector {
        async fn record_log_event(&self, _entry: &LogEntry) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_collect_log_success() {
        let repository = Arc::new(MockLogRepository::new());
        let metrics = Arc::new(MockMetricsCollector::new());
        let id_generator = Arc::new(UuidGenerator::new());

        let usecase = CollectLogUsecase::new(repository.clone(), metrics, id_generator);

        let request = CollectLogRequest::new(
            Utc::now(),
            LogLevel::Info,
            "Test message".to_string(),
            RequestId::from(Uuid::new_v4()),
            None,
            HttpContext::new("GET".to_string(), "/test".to_string(), 200, 100, None, None),
            ArchitectureLayer::Presentation,
            Operation::http_get(),
            LogMetadata::new(),
        );

        let result = usecase.collect(request).await;
        assert!(result.is_ok());

        let stored_logs = repository.stored_logs.lock().unwrap();
        assert_eq!(stored_logs.len(), 1);
        assert_eq!(stored_logs[0].message(), "Test message");
    }

    #[tokio::test]
    async fn test_collect_log_sensitive_info_rejected() {
        let repository = Arc::new(MockLogRepository::new());
        let metrics = Arc::new(MockMetricsCollector::new());
        let id_generator = Arc::new(UuidGenerator::new());

        let usecase = CollectLogUsecase::new(repository.clone(), metrics, id_generator);

        let request = CollectLogRequest::new(
            Utc::now(),
            LogLevel::Info,
            "User password is secret123".to_string(), // Contains sensitive info
            RequestId::from(Uuid::new_v4()),
            None,
            HttpContext::new("GET".to_string(), "/test".to_string(), 200, 100, None, None),
            ArchitectureLayer::Presentation,
            Operation::http_get(),
            LogMetadata::new(),
        );

        let result = usecase.collect(request).await;
        assert!(result.is_err());

        let stored_logs = repository.stored_logs.lock().unwrap();
        assert_eq!(stored_logs.len(), 0); // Should not store sensitive logs
    }
}