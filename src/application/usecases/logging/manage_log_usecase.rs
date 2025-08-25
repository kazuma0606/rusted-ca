use std::sync::Arc;

use crate::domain::{
    repository::log_repository::{LogRepositoryInterface, LogSearchCriteria},
    value_object::{analysis_status::AnalysisStatus, log_id::LogId},
};
use crate::shared::error::application_error::ApplicationError;

pub type ApplicationResult<T> = Result<T, ApplicationError>;

pub struct ManageLogUsecase {
    log_repository: Arc<dyn LogRepositoryInterface>,
}

impl ManageLogUsecase {
    pub fn new(log_repository: Arc<dyn LogRepositoryInterface>) -> Self {
        Self { log_repository }
    }

    pub async fn update_analysis_status(
        &self,
        log_id: &LogId,
        new_status: AnalysisStatus,
    ) -> ApplicationResult<()> {
        // Validate status transition if needed
        if let Some(current_entry) = self
            .log_repository
            .find_by_id(&log_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
        {
            let current_status = current_entry.analysis_status();
            if !current_status.can_transition_to(&new_status) {
                return Err(ApplicationError::ValidationError(format!(
                    "Cannot transition from {:?} to {:?}",
                    current_status, new_status
                )));
            }
        } else {
            return Err(ApplicationError::ResourceNotFound(format!(
                "Log entry not found: {}",
                log_id
            )));
        }

        self.log_repository
            .update_analysis_status(&log_id, new_status)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn add_tags(&self, log_id: &LogId, tags: Vec<String>) -> ApplicationResult<()> {
        // Validate tags
        for tag in &tags {
            self.validate_tag(tag)?;
        }

        // Check if log exists
        if self
            .log_repository
            .find_by_id(&log_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .is_none()
        {
            return Err(ApplicationError::ResourceNotFound(format!(
                "Log entry not found: {}",
                log_id
            )));
        }

        self.log_repository
            .add_tags(&log_id, tags)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn remove_tag(&self, log_id: &LogId, tag: &str) -> ApplicationResult<()> {
        // Check if log exists
        if self
            .log_repository
            .find_by_id(&log_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .is_none()
        {
            return Err(ApplicationError::ResourceNotFound(format!(
                "Log entry not found: {}",
                log_id
            )));
        }

        self.log_repository
            .remove_tag(&log_id, tag)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn add_related_log(
        &self,
        log_id: LogId,
        related_log_id: LogId,
    ) -> ApplicationResult<()> {
        // Validate that both logs exist
        let log_exists = self
            .log_repository
            .find_by_id(&log_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .is_some();

        let related_exists = self
            .log_repository
            .find_by_id(&related_log_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .is_some();

        if !log_exists {
            return Err(ApplicationError::ResourceNotFound(format!(
                "Log entry not found: {}",
                log_id
            )));
        }

        if !related_exists {
            return Err(ApplicationError::ResourceNotFound(format!(
                "Related log entry not found: {}",
                related_log_id
            )));
        }

        // Prevent self-referencing
        if log_id == related_log_id {
            return Err(ApplicationError::ValidationError(
                "Log entry cannot be related to itself".to_string(),
            ));
        }

        self.log_repository
            .add_related_log(&log_id, related_log_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn delete_logs_older_than(
        &self,
        cutoff_time: chrono::DateTime<chrono::Utc>,
    ) -> ApplicationResult<usize> {
        let now = chrono::Utc::now();
        if cutoff_time >= now {
            return Err(ApplicationError::ValidationError(
                "Cutoff time must be in the past".to_string(),
            ));
        }

        // Ensure we don't delete logs newer than 24 hours by accident
        let min_cutoff = now - chrono::Duration::hours(24);
        if cutoff_time > min_cutoff {
            return Err(ApplicationError::ValidationError(
                "Cannot delete logs newer than 24 hours".to_string(),
            ));
        }

        self.log_repository
            .delete_older_than(cutoff_time)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn count_logs_older_than(
        &self,
        cutoff_time: chrono::DateTime<chrono::Utc>,
    ) -> ApplicationResult<usize> {
        use crate::domain::repository::log_repository::LogSearchCriteria;

        let criteria = LogSearchCriteria::new().with_end_time(cutoff_time);

        self.log_repository
            .count_by_criteria(&criteria)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))
    }

    pub async fn start_log_analysis(&self, log_id: LogId) -> ApplicationResult<()> {
        // Update status to Processing
        self.update_analysis_status(&log_id, AnalysisStatus::Processing)
            .await?;

        // In a real implementation, this would:
        // 1. Queue the log for background analysis
        // 2. Apply ML models for anomaly detection
        // 3. Perform pattern matching
        // 4. Generate insights and recommendations
        // 5. Update status to Completed or Failed

        // For now, we'll just simulate a successful start
        println!("Started analysis for log: {}", log_id);

        Ok(())
    }

    pub async fn batch_tag_logs(
        &self,
        log_ids: Vec<LogId>,
        tags: Vec<String>,
    ) -> ApplicationResult<Vec<LogId>> {
        // Validate tags
        for tag in &tags {
            self.validate_tag(tag)?;
        }

        let mut successful_ids = Vec::new();
        let mut failed_ids = Vec::new();

        for log_id in log_ids {
            match self.add_tags(&log_id, tags.clone()).await {
                Ok(()) => successful_ids.push(log_id),
                Err(_) => failed_ids.push(log_id),
            }
        }

        if !failed_ids.is_empty() {
            return Err(ApplicationError::ValidationError(format!(
                "Failed to tag {} logs: {:?}",
                failed_ids.len(),
                failed_ids
            )));
        }

        Ok(successful_ids)
    }

    pub async fn get_log_statistics(&self) -> ApplicationResult<LogStatistics> {
        // This would typically use aggregation queries
        // For now, we'll return a placeholder
        Ok(LogStatistics {
            total_logs: 0,
            logs_by_level: std::collections::HashMap::new(),
            logs_by_layer: std::collections::HashMap::new(),
            pending_analysis: 0,
            completed_analysis: 0,
            failed_analysis: 0,
        })
    }

    fn validate_tag(&self, tag: &str) -> ApplicationResult<()> {
        if tag.trim().is_empty() {
            return Err(ApplicationError::ValidationError(
                "Tag cannot be empty".to_string(),
            ));
        }

        if tag.len() > 50 {
            return Err(ApplicationError::ValidationError(
                "Tag cannot exceed 50 characters".to_string(),
            ));
        }

        // Only allow alphanumeric, underscore, and hyphen
        if !tag
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ApplicationError::ValidationError(
                "Tag can only contain alphanumeric characters, underscore, and hyphen".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LogStatistics {
    pub total_logs: usize,
    pub logs_by_level:
        std::collections::HashMap<crate::domain::value_object::log_level::LogLevel, usize>,
    pub logs_by_layer: std::collections::HashMap<
        crate::domain::value_object::architecture_layer::ArchitectureLayer,
        usize,
    >,
    pub pending_analysis: usize,
    pub completed_analysis: usize,
    pub failed_analysis: usize,
}

impl LogStatistics {
    pub fn error_rate(&self) -> f64 {
        if self.total_logs == 0 {
            return 0.0;
        }

        let error_count = self
            .logs_by_level
            .get(&crate::domain::value_object::log_level::LogLevel::Error)
            .unwrap_or(&0)
            + self
                .logs_by_level
                .get(&crate::domain::value_object::log_level::LogLevel::Critical)
                .unwrap_or(&0);

        error_count as f64 / self.total_logs as f64
    }

    pub fn analysis_completion_rate(&self) -> f64 {
        let total_analyzed = self.pending_analysis + self.completed_analysis + self.failed_analysis;
        if total_analyzed == 0 {
            return 0.0;
        }

        self.completed_analysis as f64 / total_analyzed as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_validate_tag() {
        let usecase = ManageLogUsecase::new(Arc::new(MockRepository));

        assert!(usecase.validate_tag("valid-tag").is_ok());
        assert!(usecase.validate_tag("valid_tag_123").is_ok());
        assert!(usecase.validate_tag("").is_err());
        assert!(usecase.validate_tag("invalid tag!").is_err());
        assert!(usecase.validate_tag(&"x".repeat(51)).is_err());
    }

    #[tokio::test]
    async fn test_add_related_log_self_reference() {
        let usecase = ManageLogUsecase::new(Arc::new(MockRepository));
        let log_id = LogId::from(Uuid::new_v4());

        let result = usecase.add_related_log(log_id.clone(), log_id).await;
        assert!(result.is_err());
    }

    // Mock repository for testing
    struct MockRepository;

    #[async_trait::async_trait]
    impl LogRepositoryInterface for MockRepository {
        async fn store(
            &self,
            _entry: &crate::domain::entity::log_entry::LogEntry,
        ) -> Result<(), crate::shared::error::domain_error::DomainError> {
            Ok(())
        }

        async fn find_by_id(
            &self,
            _id: &LogId,
        ) -> Result<
            Option<crate::domain::entity::log_entry::LogEntry>,
            crate::shared::error::domain_error::DomainError,
        > {
            // Return None to simulate not found for most tests
            Ok(None)
        }

        async fn find_by_criteria(
            &self,
            _criteria: &crate::domain::repository::log_repository::LogSearchCriteria,
        ) -> Result<
            crate::domain::repository::log_repository::LogSearchResult,
            crate::shared::error::domain_error::DomainError,
        > {
            Ok(crate::domain::repository::log_repository::LogSearchResult::simple(Vec::new()))
        }

        async fn update_analysis_status(
            &self,
            _id: &LogId,
            _status: AnalysisStatus,
        ) -> Result<(), crate::shared::error::domain_error::DomainError> {
            Ok(())
        }

        async fn add_tags(
            &self,
            _id: &LogId,
            _tags: Vec<String>,
        ) -> Result<(), crate::shared::error::domain_error::DomainError> {
            Ok(())
        }

        async fn remove_tag(
            &self,
            _id: &LogId,
            _tag: &str,
        ) -> Result<(), crate::shared::error::domain_error::DomainError> {
            Ok(())
        }

        async fn add_related_log(
            &self,
            _id: &LogId,
            _related_id: LogId,
        ) -> Result<(), crate::shared::error::domain_error::DomainError> {
            Ok(())
        }

        async fn count_by_criteria(
            &self,
            _criteria: &crate::domain::repository::log_repository::LogSearchCriteria,
        ) -> Result<usize, crate::shared::error::domain_error::DomainError> {
            Ok(0)
        }

        async fn delete_older_than(
            &self,
            _cutoff_time: chrono::DateTime<chrono::Utc>,
        ) -> Result<usize, crate::shared::error::domain_error::DomainError> {
            Ok(0)
        }

        async fn find_by_request_id(
            &self,
            _request_id: &crate::domain::value_object::request_id::RequestId,
        ) -> Result<
            Vec<crate::domain::entity::log_entry::LogEntry>,
            crate::shared::error::domain_error::DomainError,
        > {
            Ok(Vec::new())
        }

        async fn find_errors_in_range(
            &self,
            _start: chrono::DateTime<chrono::Utc>,
            _end: chrono::DateTime<chrono::Utc>,
        ) -> Result<
            Vec<crate::domain::entity::log_entry::LogEntry>,
            crate::shared::error::domain_error::DomainError,
        > {
            Ok(Vec::new())
        }

        async fn get_performance_metrics(
            &self,
            _start: chrono::DateTime<chrono::Utc>,
            _end: chrono::DateTime<chrono::Utc>,
        ) -> Result<
            crate::domain::repository::log_repository::PerformanceMetrics,
            crate::shared::error::domain_error::DomainError,
        > {
            Ok(crate::domain::repository::log_repository::PerformanceMetrics::new())
        }
    }
}
