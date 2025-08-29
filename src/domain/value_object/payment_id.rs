//domain/value_object/payment_id.rs
// PaymentId バリューオブジェクト
// 2025/8/28

use crate::shared::error::domain_error::{DomainError, DomainResult};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaymentId {
    value: String,
}

impl PaymentId {
    pub fn new(value: String) -> DomainResult<Self> {
        if value.is_empty() {
            return Err(DomainError::InvalidPaymentId {
                reason: "Payment ID cannot be empty".to_string(),
            });
        }

        // Validate UUID format
        if Uuid::parse_str(&value).is_err() {
            return Err(DomainError::InvalidPaymentId {
                reason: "Payment ID must be a valid UUID".to_string(),
            });
        }

        Ok(Self { value })
    }

    pub fn generate() -> Self {
        Self {
            value: Uuid::new_v4().to_string(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for PaymentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<PaymentId> for String {
    fn from(payment_id: PaymentId) -> Self {
        payment_id.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_id_creation_success() {
        let uuid_str = Uuid::new_v4().to_string();
        let payment_id = PaymentId::new(uuid_str.clone()).unwrap();
        assert_eq!(payment_id.value(), uuid_str);
    }

    #[test]
    fn test_payment_id_empty_fails() {
        let result = PaymentId::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_id_invalid_uuid_fails() {
        let result = PaymentId::new("invalid-uuid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_id_generate() {
        let payment_id = PaymentId::generate();
        assert!(Uuid::parse_str(payment_id.value()).is_ok());
    }

    #[test]
    fn test_payment_id_display() {
        let payment_id = PaymentId::generate();
        let displayed = format!("{}", payment_id);
        assert_eq!(displayed, payment_id.value());
    }
}