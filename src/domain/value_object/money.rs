//domain/value_object/money.rs
// Money バリューオブジェクト
// 2025/8/28

use crate::shared::error::domain_error::{DomainError, DomainResult};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money {
    /// Amount in smallest currency unit (cents for USD, yen for JPY)
    amount_cents: i64,
    /// ISO 4217 currency code
    currency_code: String,
}

impl Money {
    pub fn new(amount_cents: i64, currency_code: String) -> DomainResult<Self> {
        if currency_code.is_empty() {
            return Err(DomainError::InvalidMoney {
                reason: "Currency code cannot be empty".to_string(),
            });
        }
        
        if currency_code.len() != 3 {
            return Err(DomainError::InvalidMoney {
                reason: "Currency code must be exactly 3 characters".to_string(),
            });
        }

        // Allow negative amounts for accounting purposes (debits)
        Ok(Self {
            amount_cents,
            currency_code: currency_code.to_uppercase(),
        })
    }

    /// Create money from major currency units (e.g., dollars, yen)
    pub fn from_major_units(amount: f64, currency_code: String) -> DomainResult<Self> {
        let cents = match currency_code.to_uppercase().as_str() {
            "JPY" | "KRW" => amount as i64, // No fractional units
            _ => (amount * 100.0).round() as i64, // Most currencies have cents
        };
        
        Self::new(cents, currency_code)
    }

    /// Create zero amount
    pub fn zero(currency_code: String) -> DomainResult<Self> {
        Self::new(0, currency_code)
    }

    pub fn amount_cents(&self) -> i64 {
        self.amount_cents
    }

    pub fn currency_code(&self) -> &str {
        &self.currency_code
    }

    /// Get amount in major currency units (e.g., dollars, yen)
    pub fn to_major_units(&self) -> f64 {
        match self.currency_code.as_str() {
            "JPY" | "KRW" => self.amount_cents as f64,
            _ => self.amount_cents as f64 / 100.0,
        }
    }

    /// Add money (same currency only)
    pub fn add(&self, other: &Money) -> DomainResult<Money> {
        if self.currency_code != other.currency_code {
            return Err(DomainError::CurrencyMismatch {
                expected: self.currency_code.clone(),
                actual: other.currency_code.clone(),
            });
        }

        Ok(Money {
            amount_cents: self.amount_cents + other.amount_cents,
            currency_code: self.currency_code.clone(),
        })
    }

    /// Subtract money (same currency only)
    pub fn subtract(&self, other: &Money) -> DomainResult<Money> {
        if self.currency_code != other.currency_code {
            return Err(DomainError::CurrencyMismatch {
                expected: self.currency_code.clone(),
                actual: other.currency_code.clone(),
            });
        }

        Ok(Money {
            amount_cents: self.amount_cents - other.amount_cents,
            currency_code: self.currency_code.clone(),
        })
    }

    /// Check if amount is positive
    pub fn is_positive(&self) -> bool {
        self.amount_cents > 0
    }

    /// Check if amount is negative
    pub fn is_negative(&self) -> bool {
        self.amount_cents < 0
    }

    /// Check if amount is zero
    pub fn is_zero(&self) -> bool {
        self.amount_cents == 0
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.currency_code.as_str() {
            "JPY" => write!(f, "¥{}", self.amount_cents),
            "USD" => write!(f, "${:.2}", self.to_major_units()),
            "EUR" => write!(f, "€{:.2}", self.to_major_units()),
            _ => write!(f, "{:.2} {}", self.to_major_units(), self.currency_code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_creation_success() {
        let money = Money::new(100, "USD".to_string()).unwrap();
        assert_eq!(money.amount_cents(), 100);
        assert_eq!(money.currency_code(), "USD");
    }

    #[test]
    fn test_money_from_major_units() {
        let money = Money::from_major_units(12.50, "USD".to_string()).unwrap();
        assert_eq!(money.amount_cents(), 1250);
        assert_eq!(money.to_major_units(), 12.50);
    }

    #[test]
    fn test_money_jpy_no_fractional() {
        let money = Money::from_major_units(1000.0, "JPY".to_string()).unwrap();
        assert_eq!(money.amount_cents(), 1000);
        assert_eq!(money.to_major_units(), 1000.0);
    }

    #[test]
    fn test_money_add_same_currency() {
        let money1 = Money::new(100, "USD".to_string()).unwrap();
        let money2 = Money::new(50, "USD".to_string()).unwrap();
        let result = money1.add(&money2).unwrap();
        assert_eq!(result.amount_cents(), 150);
    }

    #[test]
    fn test_money_add_different_currency_fails() {
        let money1 = Money::new(100, "USD".to_string()).unwrap();
        let money2 = Money::new(50, "JPY".to_string()).unwrap();
        assert!(money1.add(&money2).is_err());
    }

    #[test]
    fn test_money_subtract() {
        let money1 = Money::new(100, "USD".to_string()).unwrap();
        let money2 = Money::new(30, "USD".to_string()).unwrap();
        let result = money1.subtract(&money2).unwrap();
        assert_eq!(result.amount_cents(), 70);
    }

    #[test]
    fn test_money_display_formatting() {
        let usd = Money::new(1250, "USD".to_string()).unwrap();
        assert_eq!(format!("{}", usd), "$12.50");

        let jpy = Money::new(1000, "JPY".to_string()).unwrap();
        assert_eq!(format!("{}", jpy), "¥1000");
    }

    #[test]
    fn test_money_validation() {
        assert!(Money::new(100, "".to_string()).is_err());
        assert!(Money::new(100, "US".to_string()).is_err());
        assert!(Money::new(100, "INVALID".to_string()).is_err());
    }

    #[test]
    fn test_money_zero() {
        let zero = Money::zero("USD".to_string()).unwrap();
        assert!(zero.is_zero());
        assert!(!zero.is_positive());
        assert!(!zero.is_negative());
    }

    #[test]
    fn test_money_sign_checks() {
        let positive = Money::new(100, "USD".to_string()).unwrap();
        let negative = Money::new(-100, "USD".to_string()).unwrap();
        
        assert!(positive.is_positive());
        assert!(negative.is_negative());
    }
}