//domain/value_object/account_id.rs
// AccountId バリューオブジェクト
// 2025/8/28

use crate::shared::error::domain_error::{DomainError, DomainResult};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountId(pub String);

impl AccountId {
    pub fn new(value: String) -> DomainResult<Self> {
        if value.is_empty() {
            return Err(DomainError::InvalidValue(
                "AccountId cannot be empty".to_string(),
            ));
        }
        
        // Validate UUID format
        Uuid::parse_str(&value).map_err(|_| {
            DomainError::InvalidValue(format!("AccountId must be a valid UUID: {}", value))
        })?;
        
        Ok(Self(value))
    }

    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_id_new_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let account_id = AccountId::new(uuid_str.to_string()).unwrap();
        assert_eq!(account_id.value(), uuid_str);
    }

    #[test]
    fn test_account_id_new_empty_fails() {
        assert!(AccountId::new("".to_string()).is_err());
    }

    #[test]
    fn test_account_id_new_invalid_uuid_fails() {
        assert!(AccountId::new("not-a-uuid".to_string()).is_err());
    }

    #[test]
    fn test_account_id_generate() {
        let account_id = AccountId::generate();
        assert!(!account_id.value().is_empty());
        // Validate it's a proper UUID
        assert!(Uuid::parse_str(account_id.value()).is_ok());
    }

    #[test]
    fn test_account_id_display() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let account_id = AccountId::new(uuid_str.to_string()).unwrap();
        assert_eq!(format!("{}", account_id), uuid_str);
    }
}