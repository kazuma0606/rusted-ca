//domain/value_object/payment_method.rs
// PaymentMethod バリューオブジェクト
// 2025/8/28

use crate::shared::error::domain_error::{DomainError, DomainResult};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentMethod {
    Card,
    BankTransfer,
    DigitalWallet,
}

impl PaymentMethod {
    pub fn from_str(value: &str) -> DomainResult<Self> {
        match value.to_uppercase().as_str() {
            "CARD" => Ok(PaymentMethod::Card),
            "BANK_TRANSFER" => Ok(PaymentMethod::BankTransfer),
            "DIGITAL_WALLET" => Ok(PaymentMethod::DigitalWallet),
            _ => Err(DomainError::InvalidPaymentMethod {
                reason: format!("Invalid payment method: {}. Must be one of: CARD, BANK_TRANSFER, DIGITAL_WALLET", value),
            }),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            PaymentMethod::Card => "CARD",
            PaymentMethod::BankTransfer => "BANK_TRANSFER",
            PaymentMethod::DigitalWallet => "DIGITAL_WALLET",
        }
    }

    pub fn is_instant(&self) -> bool {
        match self {
            PaymentMethod::Card => true,
            PaymentMethod::DigitalWallet => true,
            PaymentMethod::BankTransfer => false,
        }
    }

    pub fn requires_verification(&self) -> bool {
        match self {
            PaymentMethod::Card => true,
            PaymentMethod::BankTransfer => true,
            PaymentMethod::DigitalWallet => false,
        }
    }
}

impl fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for PaymentMethod {
    fn default() -> Self {
        PaymentMethod::Card
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method_from_str_success() {
        assert_eq!(PaymentMethod::from_str("CARD").unwrap(), PaymentMethod::Card);
        assert_eq!(PaymentMethod::from_str("card").unwrap(), PaymentMethod::Card);
        assert_eq!(PaymentMethod::from_str("BANK_TRANSFER").unwrap(), PaymentMethod::BankTransfer);
        assert_eq!(PaymentMethod::from_str("DIGITAL_WALLET").unwrap(), PaymentMethod::DigitalWallet);
    }

    #[test]
    fn test_payment_method_from_str_invalid() {
        assert!(PaymentMethod::from_str("INVALID").is_err());
        assert!(PaymentMethod::from_str("").is_err());
    }

    #[test]
    fn test_payment_method_as_str() {
        assert_eq!(PaymentMethod::Card.as_str(), "CARD");
        assert_eq!(PaymentMethod::BankTransfer.as_str(), "BANK_TRANSFER");
        assert_eq!(PaymentMethod::DigitalWallet.as_str(), "DIGITAL_WALLET");
    }

    #[test]
    fn test_payment_method_is_instant() {
        assert!(PaymentMethod::Card.is_instant());
        assert!(PaymentMethod::DigitalWallet.is_instant());
        assert!(!PaymentMethod::BankTransfer.is_instant());
    }

    #[test]
    fn test_payment_method_requires_verification() {
        assert!(PaymentMethod::Card.requires_verification());
        assert!(PaymentMethod::BankTransfer.requires_verification());
        assert!(!PaymentMethod::DigitalWallet.requires_verification());
    }

    #[test]
    fn test_payment_method_display() {
        assert_eq!(format!("{}", PaymentMethod::Card), "CARD");
        assert_eq!(format!("{}", PaymentMethod::BankTransfer), "BANK_TRANSFER");
    }
}