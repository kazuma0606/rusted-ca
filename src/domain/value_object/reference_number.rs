//domain/value_object/reference_number.rs
// ReferenceNumber バリューオブジェクト
// 2025/8/28

use crate::shared::error::domain_error::{DomainError, DomainResult};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceNumber {
    value: String,
}

impl ReferenceNumber {
    pub fn new(value: String) -> DomainResult<Self> {
        if value.is_empty() {
            return Err(DomainError::InvalidReferenceNumber {
                reason: "Reference number cannot be empty".to_string(),
            });
        }

        if value.len() > 100 {
            return Err(DomainError::InvalidReferenceNumber {
                reason: "Reference number cannot exceed 100 characters".to_string(),
            });
        }

        // Allow only alphanumeric characters, hyphens, and underscores
        if !value.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(DomainError::InvalidReferenceNumber {
                reason: "Reference number can only contain alphanumeric characters, hyphens, and underscores".to_string(),
            });
        }

        Ok(Self { value })
    }

    /// Generate a new reference number with timestamp and random component
    pub fn generate() -> Self {
        use chrono::Utc;
        use rand::Rng;
        
        let timestamp = Utc::now().timestamp();
        let random: u32 = rand::thread_rng().gen_range(1000..9999);
        let value = format!("PAY-{}-{}", timestamp, random);
        
        Self { value }
    }

    /// Generate with custom prefix
    pub fn generate_with_prefix(prefix: &str) -> DomainResult<Self> {
        if prefix.is_empty() || prefix.len() > 10 {
            return Err(DomainError::InvalidReferenceNumber {
                reason: "Prefix must be 1-10 characters".to_string(),
            });
        }

        use chrono::Utc;
        use rand::Rng;
        
        let timestamp = Utc::now().timestamp();
        let random: u32 = rand::thread_rng().gen_range(1000..9999);
        let value = format!("{}-{}-{}", prefix.to_uppercase(), timestamp, random);
        
        Self::new(value)
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for ReferenceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<ReferenceNumber> for String {
    fn from(reference_number: ReferenceNumber) -> Self {
        reference_number.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_number_creation_success() {
        let ref_num = ReferenceNumber::new("PAY-123456789-001".to_string()).unwrap();
        assert_eq!(ref_num.value(), "PAY-123456789-001");
    }

    #[test]
    fn test_reference_number_empty_fails() {
        let result = ReferenceNumber::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_reference_number_too_long_fails() {
        let long_string = "A".repeat(101);
        let result = ReferenceNumber::new(long_string);
        assert!(result.is_err());
    }

    #[test]
    fn test_reference_number_invalid_chars_fails() {
        let result = ReferenceNumber::new("PAY-123@456".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_reference_number_valid_chars_success() {
        let ref_num = ReferenceNumber::new("PAY-123_456-789".to_string()).unwrap();
        assert_eq!(ref_num.value(), "PAY-123_456-789");
    }

    #[test]
    fn test_reference_number_generate() {
        let ref_num = ReferenceNumber::generate();
        assert!(ref_num.value().starts_with("PAY-"));
        assert!(ref_num.value().len() > 10);
    }

    #[test]
    fn test_reference_number_generate_with_prefix() {
        let ref_num = ReferenceNumber::generate_with_prefix("TEST").unwrap();
        assert!(ref_num.value().starts_with("TEST-"));
    }

    #[test]
    fn test_reference_number_generate_with_invalid_prefix() {
        let result = ReferenceNumber::generate_with_prefix("");
        assert!(result.is_err());
        
        let result = ReferenceNumber::generate_with_prefix("VERY_LONG_PREFIX");
        assert!(result.is_err());
    }

    #[test]
    fn test_reference_number_display() {
        let ref_num = ReferenceNumber::new("TEST-123".to_string()).unwrap();
        assert_eq!(format!("{}", ref_num), "TEST-123");
    }
}