//domain/value_object/merchant_name.rs
// MerchantName バリューオブジェクト
// 2025/8/28

use crate::shared::error::domain_error::{DomainError, DomainResult};

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantName(pub String);

impl MerchantName {
    pub fn new(value: String) -> DomainResult<Self> {
        if value.is_empty() {
            return Err(DomainError::InvalidValue(
                "Merchant name cannot be empty".to_string(),
            ));
        }

        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidValue(
                "Merchant name cannot be only whitespace".to_string(),
            ));
        }

        if trimmed.len() > 255 {
            return Err(DomainError::InvalidValue(
                "Merchant name cannot exceed 255 characters".to_string(),
            ));
        }

        // Business rule: Merchant name should contain valid characters
        if !Self::is_valid_merchant_name(trimmed) {
            return Err(DomainError::InvalidValue(
                "Merchant name contains invalid characters".to_string(),
            ));
        }

        Ok(Self(trimmed.to_string()))
    }

    fn is_valid_merchant_name(name: &str) -> bool {
        // Allow alphanumeric, spaces, hyphens, underscores, dots, and common business characters
        name.chars().all(|c| {
            c.is_alphanumeric() 
            || c.is_whitespace() 
            || matches!(c, '-' | '_' | '.' | '&' | ',' | '\'' | '(' | ')' | '/' | '+')
        })
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MerchantName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merchant_name_new_valid() {
        let valid_names = vec![
            "Acme Corp",
            "John's Coffee Shop",
            "Tech Solutions Inc.",
            "ABC-123 Store",
            "Business & Co.",
            "Restaurant (Downtown)",
            "Shop/Market+",
        ];

        for name in valid_names {
            assert!(MerchantName::new(name.to_string()).is_ok(), "Failed for: {}", name);
        }
    }

    #[test]
    fn test_merchant_name_new_empty_fails() {
        assert!(MerchantName::new("".to_string()).is_err());
        assert!(MerchantName::new("   ".to_string()).is_err());
        assert!(MerchantName::new("\t\n".to_string()).is_err());
    }

    #[test]
    fn test_merchant_name_too_long_fails() {
        let long_name = "a".repeat(256);
        assert!(MerchantName::new(long_name).is_err());
    }

    #[test]
    fn test_merchant_name_invalid_characters_fails() {
        let invalid_names = vec![
            "Shop@Email.com", // @ symbol not allowed
            "Store#1",        // # symbol not allowed
            "Business%",      // % symbol not allowed
            "Shop$",          // $ symbol not allowed
            "Store^",         // ^ symbol not allowed
            "Business*",      // * symbol not allowed
        ];

        for name in invalid_names {
            assert!(MerchantName::new(name.to_string()).is_err(), "Should fail for: {}", name);
        }
    }

    #[test]
    fn test_merchant_name_trims_whitespace() {
        let name = MerchantName::new("  Acme Corp  ".to_string()).unwrap();
        assert_eq!(name.value(), "Acme Corp");
    }

    #[test]
    fn test_merchant_name_display() {
        let name = MerchantName::new("Test Store".to_string()).unwrap();
        assert_eq!(format!("{}", name), "Test Store");
    }

    #[test]
    fn test_merchant_name_value() {
        let name = MerchantName::new("Test Store".to_string()).unwrap();
        assert_eq!(name.value(), "Test Store");
    }
}