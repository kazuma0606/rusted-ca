use serde::{Deserialize, Serialize};
use std::fmt;

use crate::shared::error::domain_error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation(String);

impl Operation {
    pub fn new(name: &str) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::InvalidValue(
                "Operation name cannot be empty".to_string(),
            ));
        }

        if name.len() > 100 {
            return Err(DomainError::InvalidValue(
                "Operation name cannot exceed 100 characters".to_string(),
            ));
        }

        Ok(Self(name.trim().to_string()))
    }

    pub fn from_string(s: &str) -> Result<Self, DomainError> {
        Self::new(s)
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    // Common operation constructors
    pub fn http_get() -> Self {
        Self("HTTP_GET".to_string())
    }

    pub fn http_post() -> Self {
        Self("HTTP_POST".to_string())
    }

    pub fn http_put() -> Self {
        Self("HTTP_PUT".to_string())
    }

    pub fn http_delete() -> Self {
        Self("HTTP_DELETE".to_string())
    }

    pub fn usecase(name: &str) -> Result<Self, DomainError> {
        Self::new(&format!("USECASE_{}", name.to_uppercase()))
    }

    pub fn repository(name: &str) -> Result<Self, DomainError> {
        Self::new(&format!("REPOSITORY_{}", name.to_uppercase()))
    }

    pub fn domain_service(name: &str) -> Result<Self, DomainError> {
        Self::new(&format!("DOMAIN_SERVICE_{}", name.to_uppercase()))
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_new() {
        let op = Operation::new("create_user").unwrap();
        assert_eq!(op.value(), "create_user");
    }

    #[test]
    fn test_operation_empty() {
        assert!(Operation::new("").is_err());
        assert!(Operation::new("   ").is_err());
    }

    #[test]
    fn test_operation_too_long() {
        let long_name = "a".repeat(101);
        assert!(Operation::new(&long_name).is_err());
    }

    #[test]
    fn test_operation_trim() {
        let op = Operation::new("  create_user  ").unwrap();
        assert_eq!(op.value(), "create_user");
    }

    #[test]
    fn test_operation_constructors() {
        assert_eq!(Operation::http_get().value(), "HTTP_GET");
        assert_eq!(Operation::http_post().value(), "HTTP_POST");
        
        let usecase_op = Operation::usecase("create_user").unwrap();
        assert_eq!(usecase_op.value(), "USECASE_CREATE_USER");
        
        let repo_op = Operation::repository("user_save").unwrap();
        assert_eq!(repo_op.value(), "REPOSITORY_USER_SAVE");
    }
}