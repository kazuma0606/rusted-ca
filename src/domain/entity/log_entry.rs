use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_object::{
    analysis_status::AnalysisStatus,
    architecture_layer::ArchitectureLayer,
    http_context::HttpContext,
    log_id::LogId,
    log_level::LogLevel,
    log_metadata::LogMetadata,
    operation::Operation,
    request_id::RequestId,
    user_context::UserContext,
};
use crate::shared::error::domain_error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    id: LogId,
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    request_id: RequestId,
    user_context: Option<UserContext>,
    http_context: HttpContext,
    architecture_layer: ArchitectureLayer,
    operation: Operation,
    metadata: LogMetadata,
    tags: Vec<String>,
    analysis_status: AnalysisStatus,
    related_log_ids: Vec<LogId>,
}

impl LogEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: LogId,
        timestamp: DateTime<Utc>,
        level: LogLevel,
        message: String,
        request_id: RequestId,
        user_context: Option<UserContext>,
        http_context: HttpContext,
        architecture_layer: ArchitectureLayer,
        operation: Operation,
        metadata: LogMetadata,
    ) -> Result<Self, DomainError> {
        Self::validate_message(&message)?;

        Ok(Self {
            id,
            timestamp,
            level,
            message,
            request_id,
            user_context,
            http_context,
            architecture_layer,
            operation,
            metadata,
            tags: Vec::new(),
            analysis_status: AnalysisStatus::default(),
            related_log_ids: Vec::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstruct(
        id: LogId,
        timestamp: DateTime<Utc>,
        level: LogLevel,
        message: String,
        request_id: RequestId,
        user_context: Option<UserContext>,
        http_context: HttpContext,
        architecture_layer: ArchitectureLayer,
        operation: Operation,
        metadata: LogMetadata,
        tags: Vec<String>,
        analysis_status: AnalysisStatus,
        related_log_ids: Vec<LogId>,
    ) -> Result<Self, DomainError> {
        Self::validate_message(&message)?;
        Self::validate_tags(&tags)?;

        Ok(Self {
            id,
            timestamp,
            level,
            message,
            request_id,
            user_context,
            http_context,
            architecture_layer,
            operation,
            metadata,
            tags,
            analysis_status,
            related_log_ids,
        })
    }

    // Getters
    pub fn id(&self) -> &LogId {
        &self.id
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn level(&self) -> LogLevel {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    pub fn user_context(&self) -> &Option<UserContext> {
        &self.user_context
    }

    pub fn http_context(&self) -> &HttpContext {
        &self.http_context
    }

    pub fn architecture_layer(&self) -> ArchitectureLayer {
        self.architecture_layer
    }

    pub fn operation(&self) -> &Operation {
        &self.operation
    }

    pub fn metadata(&self) -> &LogMetadata {
        &self.metadata
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn analysis_status(&self) -> AnalysisStatus {
        self.analysis_status
    }

    pub fn related_log_ids(&self) -> &[LogId] {
        &self.related_log_ids
    }

    // Business methods
    pub fn update_analysis_status(&mut self, new_status: AnalysisStatus) -> Result<(), DomainError> {
        if !self.analysis_status.can_transition_to(&new_status) {
            return Err(DomainError::InvalidOperation(format!(
                "Cannot transition from {:?} to {:?}",
                self.analysis_status, new_status
            )));
        }

        self.analysis_status = new_status;
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) -> Result<(), DomainError> {
        Self::validate_tag(&tag)?;

        if self.tags.contains(&tag) {
            return Ok(()); // Tag already exists, no error
        }

        if self.tags.len() >= 50 {
            return Err(DomainError::InvalidOperation(
                "Cannot add more than 50 tags to a log entry".to_string(),
            ));
        }

        self.tags.push(tag);
        Ok(())
    }

    pub fn add_tags(&mut self, mut new_tags: Vec<String>) -> Result<(), DomainError> {
        // Validate all tags first
        for tag in &new_tags {
            Self::validate_tag(tag)?;
        }

        // Remove duplicates and existing tags
        new_tags.retain(|tag| !self.tags.contains(tag));
        
        if self.tags.len() + new_tags.len() > 50 {
            return Err(DomainError::InvalidOperation(
                "Cannot add more than 50 tags total to a log entry".to_string(),
            ));
        }

        self.tags.extend(new_tags);
        Ok(())
    }

    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn add_related_log(&mut self, log_id: LogId) -> Result<(), DomainError> {
        if self.related_log_ids.contains(&log_id) {
            return Ok(()); // Already related, no error
        }

        if log_id == self.id {
            return Err(DomainError::InvalidOperation(
                "Log entry cannot be related to itself".to_string(),
            ));
        }

        if self.related_log_ids.len() >= 20 {
            return Err(DomainError::InvalidOperation(
                "Cannot add more than 20 related log entries".to_string(),
            ));
        }

        self.related_log_ids.push(log_id);
        Ok(())
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn is_error(&self) -> bool {
        self.level.is_error_or_above()
    }

    pub fn is_slow_request(&self, threshold_ms: u64) -> bool {
        self.http_context.response_time_ms() > threshold_ms
    }

    // Validation methods
    fn validate_message(message: &str) -> Result<(), DomainError> {
        if message.trim().is_empty() {
            return Err(DomainError::InvalidValue(
                "Log message cannot be empty".to_string(),
            ));
        }

        if message.len() > 10000 {
            return Err(DomainError::InvalidValue(
                "Log message cannot exceed 10000 characters".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_tag(tag: &str) -> Result<(), DomainError> {
        if tag.trim().is_empty() {
            return Err(DomainError::InvalidValue(
                "Tag cannot be empty".to_string(),
            ));
        }

        if tag.len() > 50 {
            return Err(DomainError::InvalidValue(
                "Tag cannot exceed 50 characters".to_string(),
            ));
        }

        // Only allow alphanumeric, underscore, and hyphen
        if !tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(DomainError::InvalidValue(
                "Tag can only contain alphanumeric characters, underscore, and hyphen".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_tags(tags: &[String]) -> Result<(), DomainError> {
        if tags.len() > 50 {
            return Err(DomainError::InvalidValue(
                "Cannot have more than 50 tags".to_string(),
            ));
        }

        for tag in tags {
            Self::validate_tag(tag)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_log_entry() -> LogEntry {
        LogEntry::new(
            LogId::from(Uuid::new_v4()),
            Utc::now(),
            LogLevel::Info,
            "Test message".to_string(),
            RequestId::from(Uuid::new_v4()),
            None,
            HttpContext::new(
                "GET".to_string(),
                "/test".to_string(),
                200,
                100,
                None,
                None,
            ),
            ArchitectureLayer::Presentation,
            Operation::http_get(),
            LogMetadata::new(),
        ).unwrap()
    }

    #[test]
    fn test_log_entry_new() {
        let entry = create_test_log_entry();
        assert_eq!(entry.level(), LogLevel::Info);
        assert_eq!(entry.message(), "Test message");
        assert!(!entry.is_error());
    }

    #[test]
    fn test_empty_message() {
        let result = LogEntry::new(
            LogId::from(Uuid::new_v4()),
            Utc::now(),
            LogLevel::Info,
            "".to_string(),
            RequestId::from(Uuid::new_v4()),
            None,
            HttpContext::new(
                "GET".to_string(),
                "/test".to_string(),
                200,
                100,
                None,
                None,
            ),
            ArchitectureLayer::Presentation,
            Operation::http_get(),
            LogMetadata::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_add_tag() {
        let mut entry = create_test_log_entry();
        entry.add_tag("test-tag".to_string()).unwrap();
        assert!(entry.has_tag("test-tag"));
        assert_eq!(entry.tags().len(), 1);
    }

    #[test]
    fn test_add_duplicate_tag() {
        let mut entry = create_test_log_entry();
        entry.add_tag("test-tag".to_string()).unwrap();
        entry.add_tag("test-tag".to_string()).unwrap(); // Should not fail
        assert_eq!(entry.tags().len(), 1);
    }

    #[test]
    fn test_invalid_tag() {
        let mut entry = create_test_log_entry();
        let result = entry.add_tag("invalid tag!".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_analysis_status_transition() {
        let mut entry = create_test_log_entry();
        assert_eq!(entry.analysis_status(), AnalysisStatus::Pending);

        entry.update_analysis_status(AnalysisStatus::Processing).unwrap();
        assert_eq!(entry.analysis_status(), AnalysisStatus::Processing);

        entry.update_analysis_status(AnalysisStatus::Completed).unwrap();
        assert_eq!(entry.analysis_status(), AnalysisStatus::Completed);
    }

    #[test]
    fn test_invalid_analysis_status_transition() {
        let mut entry = create_test_log_entry();
        let result = entry.update_analysis_status(AnalysisStatus::Completed);
        assert!(result.is_err()); // Can't go directly from Pending to Completed
    }
}