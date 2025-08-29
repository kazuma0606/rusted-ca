//domain/value_object/payment_status.rs
// PaymentStatus バリューオブジェクト
// 2025/8/28

use crate::shared::error::domain_error::{DomainError, DomainResult};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Success,
    Failed,
    Cancelled,
    Refunded,
}

impl PaymentStatus {
    pub fn from_str(value: &str) -> DomainResult<Self> {
        match value.to_uppercase().as_str() {
            "PENDING" => Ok(PaymentStatus::Pending),
            "PROCESSING" => Ok(PaymentStatus::Processing),
            "SUCCESS" => Ok(PaymentStatus::Success),
            "FAILED" => Ok(PaymentStatus::Failed),
            "CANCELLED" => Ok(PaymentStatus::Cancelled),
            "REFUNDED" => Ok(PaymentStatus::Refunded),
            _ => Err(DomainError::InvalidPaymentStatus {
                reason: format!("Invalid payment status: {}. Must be one of: PENDING, PROCESSING, SUCCESS, FAILED, CANCELLED, REFUNDED", value),
            }),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            PaymentStatus::Pending => "PENDING",
            PaymentStatus::Processing => "PROCESSING",
            PaymentStatus::Success => "SUCCESS",
            PaymentStatus::Failed => "FAILED",
            PaymentStatus::Cancelled => "CANCELLED",
            PaymentStatus::Refunded => "REFUNDED",
        }
    }

    /// Check if status is final (cannot be changed)
    pub fn is_final(&self) -> bool {
        matches!(self, PaymentStatus::Success | PaymentStatus::Failed | PaymentStatus::Cancelled | PaymentStatus::Refunded)
    }

    /// Check if payment is successful
    pub fn is_successful(&self) -> bool {
        matches!(self, PaymentStatus::Success)
    }

    /// Check if payment is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self, PaymentStatus::Pending | PaymentStatus::Processing)
    }

    /// Check if payment can be cancelled
    pub fn can_be_cancelled(&self) -> bool {
        matches!(self, PaymentStatus::Pending)
    }

    /// Check if payment can be refunded
    pub fn can_be_refunded(&self) -> bool {
        matches!(self, PaymentStatus::Success)
    }

    /// Valid transitions from current status
    pub fn valid_transitions(&self) -> Vec<PaymentStatus> {
        match self {
            PaymentStatus::Pending => vec![
                PaymentStatus::Processing,
                PaymentStatus::Cancelled,
                PaymentStatus::Failed,
            ],
            PaymentStatus::Processing => vec![
                PaymentStatus::Success,
                PaymentStatus::Failed,
            ],
            PaymentStatus::Success => vec![
                PaymentStatus::Refunded,
            ],
            PaymentStatus::Failed | PaymentStatus::Cancelled | PaymentStatus::Refunded => vec![],
        }
    }

    /// Check if transition to new status is valid
    pub fn can_transition_to(&self, new_status: &PaymentStatus) -> bool {
        self.valid_transitions().contains(new_status)
    }
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for PaymentStatus {
    fn default() -> Self {
        PaymentStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_status_from_str_success() {
        assert_eq!(PaymentStatus::from_str("PENDING").unwrap(), PaymentStatus::Pending);
        assert_eq!(PaymentStatus::from_str("pending").unwrap(), PaymentStatus::Pending);
        assert_eq!(PaymentStatus::from_str("SUCCESS").unwrap(), PaymentStatus::Success);
        assert_eq!(PaymentStatus::from_str("FAILED").unwrap(), PaymentStatus::Failed);
    }

    #[test]
    fn test_payment_status_from_str_invalid() {
        assert!(PaymentStatus::from_str("INVALID").is_err());
        assert!(PaymentStatus::from_str("").is_err());
    }

    #[test]
    fn test_payment_status_is_final() {
        assert!(!PaymentStatus::Pending.is_final());
        assert!(!PaymentStatus::Processing.is_final());
        assert!(PaymentStatus::Success.is_final());
        assert!(PaymentStatus::Failed.is_final());
        assert!(PaymentStatus::Cancelled.is_final());
        assert!(PaymentStatus::Refunded.is_final());
    }

    #[test]
    fn test_payment_status_is_successful() {
        assert!(PaymentStatus::Success.is_successful());
        assert!(!PaymentStatus::Failed.is_successful());
        assert!(!PaymentStatus::Pending.is_successful());
    }

    #[test]
    fn test_payment_status_can_be_cancelled() {
        assert!(PaymentStatus::Pending.can_be_cancelled());
        assert!(!PaymentStatus::Processing.can_be_cancelled());
        assert!(!PaymentStatus::Success.can_be_cancelled());
    }

    #[test]
    fn test_payment_status_can_be_refunded() {
        assert!(PaymentStatus::Success.can_be_refunded());
        assert!(!PaymentStatus::Pending.can_be_refunded());
        assert!(!PaymentStatus::Failed.can_be_refunded());
    }

    #[test]
    fn test_payment_status_transitions() {
        assert!(PaymentStatus::Pending.can_transition_to(&PaymentStatus::Processing));
        assert!(PaymentStatus::Pending.can_transition_to(&PaymentStatus::Cancelled));
        assert!(!PaymentStatus::Pending.can_transition_to(&PaymentStatus::Success));
        
        assert!(PaymentStatus::Processing.can_transition_to(&PaymentStatus::Success));
        assert!(!PaymentStatus::Processing.can_transition_to(&PaymentStatus::Cancelled));
        
        assert!(PaymentStatus::Success.can_transition_to(&PaymentStatus::Refunded));
        assert!(!PaymentStatus::Success.can_transition_to(&PaymentStatus::Failed));
        
        // Final statuses cannot transition
        assert!(!PaymentStatus::Failed.can_transition_to(&PaymentStatus::Success));
        assert!(!PaymentStatus::Refunded.can_transition_to(&PaymentStatus::Success));
    }
}