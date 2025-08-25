use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::entity::log_entry::LogEntry;
use crate::domain::value_object::{
    analysis_status::AnalysisStatus,
    architecture_layer::ArchitectureLayer,
    log_id::LogId,
    log_level::LogLevel,
    request_id::RequestId,
};
use crate::shared::error::domain_error::DomainError;

#[derive(Debug, Clone)]
pub struct LogSearchCriteria {
    pub level: Option<LogLevel>,
    pub architecture_layer: Option<ArchitectureLayer>,
    pub request_id: Option<RequestId>,
    pub user_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub message_contains: Option<String>,
    pub tags: Option<Vec<String>>,
    pub analysis_status: Option<AnalysisStatus>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl LogSearchCriteria {
    pub fn new() -> Self {
        Self {
            level: None,
            architecture_layer: None,
            request_id: None,
            user_id: None,
            start_time: None,
            end_time: None,
            message_contains: None,
            tags: None,
            analysis_status: None,
            limit: None,
            offset: None,
        }
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_architecture_layer(mut self, layer: ArchitectureLayer) -> Self {
        self.architecture_layer = Some(layer);
        self
    }

    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn with_start_time(mut self, start: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self
    }

    pub fn with_end_time(mut self, end: DateTime<Utc>) -> Self {
        self.end_time = Some(end);
        self
    }

    pub fn with_message_contains(mut self, text: String) -> Self {
        self.message_contains = Some(text);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_analysis_status(mut self, status: AnalysisStatus) -> Self {
        self.analysis_status = Some(status);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if start > end {
                return Err(DomainError::InvalidValue(
                    "Start time cannot be after end time".to_string(),
                ));
            }
        }

        if let Some(limit) = self.limit {
            if limit == 0 || limit > 10000 {
                return Err(DomainError::InvalidValue(
                    "Limit must be between 1 and 10000".to_string(),
                ));
            }
        }

        if let Some(message) = &self.message_contains {
            if message.trim().is_empty() {
                return Err(DomainError::InvalidValue(
                    "Message search text cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for LogSearchCriteria {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct LogSearchResult {
    pub logs: Vec<LogEntry>,
    pub total_count: Option<usize>,
    pub has_more: bool,
}

impl LogSearchResult {
    pub fn new(logs: Vec<LogEntry>, total_count: Option<usize>, has_more: bool) -> Self {
        Self {
            logs,
            total_count,
            has_more,
        }
    }

    pub fn simple(logs: Vec<LogEntry>) -> Self {
        Self {
            logs,
            total_count: None,
            has_more: false,
        }
    }
}

#[async_trait]
pub trait LogRepositoryInterface: Send + Sync {
    /// Store a log entry
    async fn store(&self, entry: &LogEntry) -> Result<(), DomainError>;

    /// Find a log entry by its ID
    async fn find_by_id(&self, id: &LogId) -> Result<Option<LogEntry>, DomainError>;

    /// Find log entries by criteria
    async fn find_by_criteria(&self, criteria: &LogSearchCriteria) -> Result<LogSearchResult, DomainError>;

    /// Update the analysis status of a log entry
    async fn update_analysis_status(&self, id: &LogId, status: AnalysisStatus) -> Result<(), DomainError>;

    /// Add tags to a log entry
    async fn add_tags(&self, id: &LogId, tags: Vec<String>) -> Result<(), DomainError>;

    /// Remove a tag from a log entry
    async fn remove_tag(&self, id: &LogId, tag: &str) -> Result<(), DomainError>;

    /// Add a related log ID to a log entry
    async fn add_related_log(&self, id: &LogId, related_id: LogId) -> Result<(), DomainError>;

    /// Count log entries matching criteria
    async fn count_by_criteria(&self, criteria: &LogSearchCriteria) -> Result<usize, DomainError>;

    /// Delete old log entries (for log rotation)
    async fn delete_older_than(&self, cutoff_time: DateTime<Utc>) -> Result<usize, DomainError>;

    /// Get log entries by request ID (for request tracing)
    async fn find_by_request_id(&self, request_id: &RequestId) -> Result<Vec<LogEntry>, DomainError>;

    /// Get error log entries within a time range (for monitoring)
    async fn find_errors_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LogEntry>, DomainError>;

    /// Get performance metrics for a time range
    async fn get_performance_metrics(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<PerformanceMetrics, DomainError>;
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_requests: usize,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub p95_response_time_ms: u64,
    pub error_rate: f64,
    pub requests_by_layer: std::collections::HashMap<ArchitectureLayer, usize>,
    pub requests_by_status_code: std::collections::HashMap<u16, usize>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            avg_response_time_ms: 0.0,
            min_response_time_ms: 0,
            max_response_time_ms: 0,
            p95_response_time_ms: 0,
            error_rate: 0.0,
            requests_by_layer: std::collections::HashMap::new(),
            requests_by_status_code: std::collections::HashMap::new(),
        }
    }

    pub fn is_healthy(&self, error_rate_threshold: f64, avg_response_threshold_ms: f64) -> bool {
        self.error_rate <= error_rate_threshold && self.avg_response_time_ms <= avg_response_threshold_ms
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_log_search_criteria_validation() {
        let criteria = LogSearchCriteria::new();
        assert!(criteria.validate().is_ok());

        let invalid_time_criteria = LogSearchCriteria::new()
            .with_time_range(Utc::now(), Utc::now() - chrono::Duration::hours(1));
        assert!(invalid_time_criteria.validate().is_err());

        let invalid_limit_criteria = LogSearchCriteria::new().with_limit(0);
        assert!(invalid_limit_criteria.validate().is_err());

        let invalid_message_criteria = LogSearchCriteria::new().with_message_contains("".to_string());
        assert!(invalid_message_criteria.validate().is_err());
    }

    #[test]
    fn test_log_search_criteria_builder() {
        let request_id = RequestId::from(Uuid::new_v4());
        let criteria = LogSearchCriteria::new()
            .with_level(LogLevel::Error)
            .with_architecture_layer(ArchitectureLayer::Application)
            .with_request_id(request_id.clone())
            .with_limit(100);

        assert_eq!(criteria.level, Some(LogLevel::Error));
        assert_eq!(criteria.architecture_layer, Some(ArchitectureLayer::Application));
        assert_eq!(criteria.request_id, Some(request_id));
        assert_eq!(criteria.limit, Some(100));
    }

    #[test]
    fn test_performance_metrics_healthy() {
        let mut metrics = PerformanceMetrics::new();
        metrics.error_rate = 0.01; // 1%
        metrics.avg_response_time_ms = 100.0;

        assert!(metrics.is_healthy(0.05, 200.0)); // Healthy
        assert!(!metrics.is_healthy(0.005, 200.0)); // Too many errors
        assert!(!metrics.is_healthy(0.05, 50.0)); // Too slow
    }
}