use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::shared::error::domain_error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LogId(Uuid);

impl LogId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, DomainError> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| DomainError::InvalidValue(format!("Invalid log ID: {}", s)))
    }

    pub fn value(&self) -> Uuid {
        self.0
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for LogId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for LogId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<LogId> for Uuid {
    fn from(log_id: LogId) -> Self {
        log_id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_id_generate() {
        let id1 = LogId::generate();
        let id2 = LogId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_log_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let log_id = LogId::from_string(uuid_str).unwrap();
        assert_eq!(log_id.to_string(), uuid_str);
    }

    #[test]
    fn test_log_id_invalid_string() {
        let result = LogId::from_string("invalid-uuid");
        assert!(result.is_err());
    }
}