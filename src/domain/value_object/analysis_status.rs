use serde::{Deserialize, Serialize};
use std::fmt;

use crate::shared::error::domain_error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl AnalysisStatus {
    pub fn from_string(s: &str) -> Result<Self, DomainError> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(DomainError::InvalidValue(format!(
                "Invalid analysis status: {}",
                s
            ))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Pending => "PENDING".to_string(),
            Self::Processing => "PROCESSING".to_string(),
            Self::Completed => "COMPLETED".to_string(),
            Self::Failed => "FAILED".to_string(),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    pub fn can_transition_to(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Pending, Self::Processing) => true,
            (Self::Processing, Self::Completed | Self::Failed) => true,
            (Self::Failed, Self::Processing) => true, // Retry allowed
            _ => false,
        }
    }
}

impl fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for AnalysisStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_status_from_string() {
        assert_eq!(
            AnalysisStatus::from_string("pending").unwrap(),
            AnalysisStatus::Pending
        );
        assert_eq!(
            AnalysisStatus::from_string("PROCESSING").unwrap(),
            AnalysisStatus::Processing
        );
        assert_eq!(
            AnalysisStatus::from_string("Completed").unwrap(),
            AnalysisStatus::Completed
        );
        assert_eq!(
            AnalysisStatus::from_string("failed").unwrap(),
            AnalysisStatus::Failed
        );
    }

    #[test]
    fn test_analysis_status_invalid() {
        assert!(AnalysisStatus::from_string("invalid").is_err());
    }

    #[test]
    fn test_is_terminal() {
        assert!(!AnalysisStatus::Pending.is_terminal());
        assert!(!AnalysisStatus::Processing.is_terminal());
        assert!(AnalysisStatus::Completed.is_terminal());
        assert!(AnalysisStatus::Failed.is_terminal());
    }

    #[test]
    fn test_can_transition_to() {
        let pending = AnalysisStatus::Pending;
        let processing = AnalysisStatus::Processing;
        let completed = AnalysisStatus::Completed;
        let failed = AnalysisStatus::Failed;

        assert!(pending.can_transition_to(&processing));
        assert!(!pending.can_transition_to(&completed));
        assert!(!pending.can_transition_to(&failed));

        assert!(processing.can_transition_to(&completed));
        assert!(processing.can_transition_to(&failed));
        assert!(!processing.can_transition_to(&pending));

        assert!(!completed.can_transition_to(&pending));
        assert!(!completed.can_transition_to(&processing));
        assert!(!completed.can_transition_to(&failed));

        assert!(failed.can_transition_to(&processing)); // Retry allowed
        assert!(!failed.can_transition_to(&pending));
        assert!(!failed.can_transition_to(&completed));
    }
}