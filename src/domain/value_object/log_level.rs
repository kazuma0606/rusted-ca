use serde::{Deserialize, Serialize};
use std::fmt;

use crate::shared::error::domain_error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl LogLevel {
    pub fn from_string(s: &str) -> Result<Self, DomainError> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            "critical" | "fatal" => Ok(Self::Critical),
            _ => Err(DomainError::InvalidValue(format!("Invalid log level: {}", s))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Debug => "DEBUG".to_string(),
            Self::Info => "INFO".to_string(),
            Self::Warn => "WARN".to_string(),
            Self::Error => "ERROR".to_string(),
            Self::Critical => "CRITICAL".to_string(),
        }
    }

    pub fn is_error_or_above(&self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }

    pub fn is_warn_or_above(&self) -> bool {
        matches!(self, Self::Warn | Self::Error | Self::Critical)
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_string() {
        assert_eq!(LogLevel::from_string("debug").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_string("INFO").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_string("Warning").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_string("error").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_string("CRITICAL").unwrap(), LogLevel::Critical);
    }

    #[test]
    fn test_log_level_invalid() {
        assert!(LogLevel::from_string("invalid").is_err());
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Critical);
    }

    #[test]
    fn test_error_level_checks() {
        assert!(!LogLevel::Debug.is_error_or_above());
        assert!(!LogLevel::Info.is_error_or_above());
        assert!(!LogLevel::Warn.is_error_or_above());
        assert!(LogLevel::Error.is_error_or_above());
        assert!(LogLevel::Critical.is_error_or_above());
    }
}