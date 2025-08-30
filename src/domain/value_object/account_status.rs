//domain/value_object/account_status.rs
// AccountStatus バリューオブジェクト
// 2025/8/28

use crate::shared::error::domain_error::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountStatus {
    Active,
    Suspended,
    Closed,
}

impl AccountStatus {
    pub fn new(value: String) -> DomainResult<Self> {
        match value.to_uppercase().as_str() {
            "ACTIVE" => Ok(Self::Active),
            "SUSPENDED" => Ok(Self::Suspended),
            "CLOSED" => Ok(Self::Closed),
            _ => Err(DomainError::InvalidValue(
                format!("Invalid account status: {}. Must be ACTIVE, SUSPENDED, or CLOSED", value)
            )),
        }
    }

    pub fn from_string(value: &str) -> DomainResult<Self> {
        Self::new(value.to_string())
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Suspended => "SUSPENDED",
            Self::Closed => "CLOSED",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn is_suspended(&self) -> bool {
        matches!(self, Self::Suspended)
    }

    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }
}

impl Default for AccountStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<AccountStatus> for String {
    fn from(status: AccountStatus) -> String {
        status.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_status_new_valid() {
        assert_eq!(AccountStatus::new("ACTIVE".to_string()).unwrap(), AccountStatus::Active);
        assert_eq!(AccountStatus::new("active".to_string()).unwrap(), AccountStatus::Active);
        assert_eq!(AccountStatus::new("SUSPENDED".to_string()).unwrap(), AccountStatus::Suspended);
        assert_eq!(AccountStatus::new("CLOSED".to_string()).unwrap(), AccountStatus::Closed);
    }

    #[test]
    fn test_account_status_new_invalid() {
        assert!(AccountStatus::new("INVALID".to_string()).is_err());
        assert!(AccountStatus::new("".to_string()).is_err());
        assert!(AccountStatus::new("PENDING".to_string()).is_err());
    }

    #[test]
    fn test_account_status_as_str() {
        assert_eq!(AccountStatus::Active.as_str(), "ACTIVE");
        assert_eq!(AccountStatus::Suspended.as_str(), "SUSPENDED");
        assert_eq!(AccountStatus::Closed.as_str(), "CLOSED");
    }

    #[test]
    fn test_account_status_predicates() {
        let active = AccountStatus::Active;
        let suspended = AccountStatus::Suspended;
        let closed = AccountStatus::Closed;

        assert!(active.is_active());
        assert!(!active.is_suspended());
        assert!(!active.is_closed());

        assert!(!suspended.is_active());
        assert!(suspended.is_suspended());
        assert!(!suspended.is_closed());

        assert!(!closed.is_active());
        assert!(!closed.is_suspended());
        assert!(closed.is_closed());
    }

    #[test]
    fn test_account_status_default() {
        assert_eq!(AccountStatus::default(), AccountStatus::Active);
    }

    #[test]
    fn test_account_status_display() {
        assert_eq!(format!("{}", AccountStatus::Active), "ACTIVE");
        assert_eq!(format!("{}", AccountStatus::Suspended), "SUSPENDED");
        assert_eq!(format!("{}", AccountStatus::Closed), "CLOSED");
    }
}