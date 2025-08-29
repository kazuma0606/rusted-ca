//domain/entity/payment.rs
// Payment エンティティ + ビジネスロジック
// 2025/8/28

use crate::domain::value_object::{
    AccountId, Money, PaymentId, PaymentMethod, PaymentStatus, ReferenceNumber,
};
use crate::shared::error::domain_error::{DomainError, DomainResult};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Payment {
    pub id: PaymentId,
    pub account_id: AccountId,
    pub amount: Money,
    pub payment_method: PaymentMethod,
    pub status: PaymentStatus,
    pub reference_number: ReferenceNumber,
    pub description: Option<String>,
    pub customer_email: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    pub fn new(
        id: PaymentId,
        account_id: AccountId,
        amount: Money,
        payment_method: PaymentMethod,
        status: PaymentStatus,
        reference_number: ReferenceNumber,
        description: Option<String>,
        customer_email: Option<String>,
        metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<Self> {
        // Business rule: Payment amount must be positive
        if !amount.is_positive() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "PositivePaymentAmount".to_string(),
                message: "Payment amount must be positive".to_string(),
            });
        }

        // Business rule: Description length limit
        if let Some(desc) = &description {
            if desc.len() > 1000 {
                return Err(DomainError::BusinessRuleViolation {
                    rule: "DescriptionLengthLimit".to_string(),
                    message: "Description cannot exceed 1000 characters".to_string(),
                });
            }
        }

        // Business rule: Customer email validation
        if let Some(email) = &customer_email {
            if email.is_empty() || !email.contains('@') {
                return Err(DomainError::BusinessRuleViolation {
                    rule: "ValidCustomerEmail".to_string(),
                    message: "Customer email must be valid".to_string(),
                });
            }
        }

        Ok(Payment {
            id,
            account_id,
            amount,
            payment_method,
            status,
            reference_number,
            description,
            customer_email,
            metadata,
            created_at,
            updated_at,
        })
    }

    /// Create a new payment
    pub fn create_new(
        account_id: AccountId,
        amount: Money,
        payment_method: PaymentMethod,
        description: Option<String>,
        customer_email: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> DomainResult<Self> {
        let id = PaymentId::generate();
        let status = PaymentStatus::default(); // Pending
        let reference_number = ReferenceNumber::generate();
        let now = Utc::now();

        Self::new(
            id,
            account_id,
            amount,
            payment_method,
            status,
            reference_number,
            description,
            customer_email,
            metadata,
            now,
            now,
        )
    }

    // Getters
    pub fn id(&self) -> &PaymentId {
        &self.id
    }

    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    pub fn amount(&self) -> &Money {
        &self.amount
    }

    pub fn payment_method(&self) -> &PaymentMethod {
        &self.payment_method
    }

    pub fn status(&self) -> &PaymentStatus {
        &self.status
    }

    pub fn reference_number(&self) -> &ReferenceNumber {
        &self.reference_number
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn customer_email(&self) -> Option<&String> {
        self.customer_email.as_ref()
    }

    pub fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    // Business operations
    /// Start payment processing
    pub fn start_processing(&mut self) -> DomainResult<()> {
        if !self.status.can_transition_to(&PaymentStatus::Processing) {
            return Err(DomainError::BusinessRuleViolation {
                rule: "InvalidStatusTransition".to_string(),
                message: format!(
                    "Cannot transition from {} to Processing",
                    self.status.as_str()
                ),
            });
        }

        self.status = PaymentStatus::Processing;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark payment as successful
    pub fn mark_successful(&mut self) -> DomainResult<()> {
        if !self.status.can_transition_to(&PaymentStatus::Success) {
            return Err(DomainError::BusinessRuleViolation {
                rule: "InvalidStatusTransition".to_string(),
                message: format!(
                    "Cannot transition from {} to Success",
                    self.status.as_str()
                ),
            });
        }

        self.status = PaymentStatus::Success;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark payment as failed
    pub fn mark_failed(&mut self, reason: String) -> DomainResult<()> {
        if !self.status.can_transition_to(&PaymentStatus::Failed) {
            return Err(DomainError::BusinessRuleViolation {
                rule: "InvalidStatusTransition".to_string(),
                message: format!(
                    "Cannot transition from {} to Failed",
                    self.status.as_str()
                ),
            });
        }

        // Add failure reason to metadata
        let mut metadata = self.metadata.clone().unwrap_or(serde_json::json!({}));
        if let serde_json::Value::Object(ref mut map) = metadata {
            map.insert("failure_reason".to_string(), serde_json::Value::String(reason));
            map.insert("failed_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
        }

        self.status = PaymentStatus::Failed;
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancel payment
    pub fn cancel(&mut self, reason: String) -> DomainResult<()> {
        if !self.status.can_transition_to(&PaymentStatus::Cancelled) {
            return Err(DomainError::BusinessRuleViolation {
                rule: "InvalidStatusTransition".to_string(),
                message: format!(
                    "Cannot transition from {} to Cancelled",
                    self.status.as_str()
                ),
            });
        }

        // Add cancellation reason to metadata
        let mut metadata = self.metadata.clone().unwrap_or(serde_json::json!({}));
        if let serde_json::Value::Object(ref mut map) = metadata {
            map.insert("cancellation_reason".to_string(), serde_json::Value::String(reason));
            map.insert("cancelled_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
        }

        self.status = PaymentStatus::Cancelled;
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Refund payment
    pub fn refund(&mut self, refund_amount: Option<&Money>, reason: String) -> DomainResult<()> {
        if !self.status.can_transition_to(&PaymentStatus::Refunded) {
            return Err(DomainError::BusinessRuleViolation {
                rule: "InvalidStatusTransition".to_string(),
                message: format!(
                    "Cannot transition from {} to Refunded",
                    self.status.as_str()
                ),
            });
        }

        // Validate refund amount if provided (partial refund)
        if let Some(amount) = refund_amount {
            if amount.currency_code() != self.amount.currency_code() {
                return Err(DomainError::CurrencyMismatch {
                    expected: self.amount.currency_code().to_string(),
                    actual: amount.currency_code().to_string(),
                });
            }

            if amount > &self.amount {
                return Err(DomainError::BusinessRuleViolation {
                    rule: "RefundAmountLimit".to_string(),
                    message: "Refund amount cannot exceed original payment amount".to_string(),
                });
            }

            if !amount.is_positive() {
                return Err(DomainError::BusinessRuleViolation {
                    rule: "PositiveRefundAmount".to_string(),
                    message: "Refund amount must be positive".to_string(),
                });
            }
        }

        // Add refund information to metadata
        let mut metadata = self.metadata.clone().unwrap_or(serde_json::json!({}));
        if let serde_json::Value::Object(ref mut map) = metadata {
            map.insert("refund_reason".to_string(), serde_json::Value::String(reason));
            map.insert("refunded_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
            
            if let Some(amount) = refund_amount {
                map.insert("refund_amount_cents".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(amount.amount_cents())
                ));
                map.insert("refund_currency".to_string(), serde_json::Value::String(
                    amount.currency_code().to_string()
                ));
            } else {
                map.insert("refund_type".to_string(), serde_json::Value::String("full".to_string()));
            }
        }

        self.status = PaymentStatus::Refunded;
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if payment can be processed
    pub fn can_be_processed(&self) -> bool {
        self.status.can_transition_to(&PaymentStatus::Processing)
    }

    /// Check if payment can be cancelled
    pub fn can_be_cancelled(&self) -> bool {
        self.status.can_be_cancelled()
    }

    /// Check if payment can be refunded
    pub fn can_be_refunded(&self) -> bool {
        self.status.can_be_refunded()
    }

    /// Check if payment is successful
    pub fn is_successful(&self) -> bool {
        self.status.is_successful()
    }

    /// Check if payment is final (no more changes allowed)
    pub fn is_final(&self) -> bool {
        self.status.is_final()
    }

    /// Update description
    pub fn update_description(&mut self, description: Option<String>) -> DomainResult<()> {
        if self.is_final() {
            return Err(DomainError::BusinessRuleViolation {
                rule: "NoModifyFinalPayment".to_string(),
                message: "Cannot modify finalized payment".to_string(),
            });
        }

        if let Some(desc) = &description {
            if desc.len() > 1000 {
                return Err(DomainError::BusinessRuleViolation {
                    rule: "DescriptionLengthLimit".to_string(),
                    message: "Description cannot exceed 1000 characters".to_string(),
                });
            }
        }

        self.description = description;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Add or update metadata
    pub fn update_metadata(&mut self, key: String, value: serde_json::Value) -> DomainResult<()> {
        if self.is_final() && key != "internal_note" {
            return Err(DomainError::BusinessRuleViolation {
                rule: "NoModifyFinalPayment".to_string(),
                message: "Cannot modify finalized payment metadata".to_string(),
            });
        }

        let mut metadata = self.metadata.clone().unwrap_or(serde_json::json!({}));
        if let serde_json::Value::Object(ref mut map) = metadata {
            map.insert(key, value);
        }

        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_object::{Email, MerchantName};

    fn create_test_payment() -> Payment {
        let account_id = AccountId::generate();
        let amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        let payment_method = PaymentMethod::Card;
        
        Payment::create_new(
            account_id,
            amount,
            payment_method,
            Some("Test payment".to_string()),
            Some("customer@example.com".to_string()),
            None,
        ).unwrap()
    }

    #[test]
    fn test_payment_creation() {
        let payment = create_test_payment();
        
        assert!(payment.amount().is_positive());
        assert_eq!(payment.status(), &PaymentStatus::Pending);
        assert_eq!(payment.description(), Some(&"Test payment".to_string()));
        assert_eq!(payment.customer_email(), Some(&"customer@example.com".to_string()));
    }

    #[test]
    fn test_payment_creation_negative_amount_fails() {
        let account_id = AccountId::generate();
        let amount = Money::from_major_units(-100.0, "USD".to_string()).unwrap();
        let payment_method = PaymentMethod::Card;
        
        let result = Payment::create_new(
            account_id,
            amount,
            payment_method,
            None,
            None,
            None,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_status_transitions() {
        let mut payment = create_test_payment();
        
        // Pending -> Processing
        assert!(payment.start_processing().is_ok());
        assert_eq!(payment.status(), &PaymentStatus::Processing);
        
        // Processing -> Success
        assert!(payment.mark_successful().is_ok());
        assert_eq!(payment.status(), &PaymentStatus::Success);
        
        // Success -> Refunded
        assert!(payment.refund(None, "Customer request".to_string()).is_ok());
        assert_eq!(payment.status(), &PaymentStatus::Refunded);
    }

    #[test]
    fn test_payment_invalid_transition() {
        let mut payment = create_test_payment();
        
        // Cannot go from Pending directly to Success
        assert!(payment.mark_successful().is_err());
    }

    #[test]
    fn test_payment_cancel() {
        let mut payment = create_test_payment();
        
        assert!(payment.cancel("User cancelled".to_string()).is_ok());
        assert_eq!(payment.status(), &PaymentStatus::Cancelled);
        
        // Check metadata contains cancellation info
        assert!(payment.metadata().is_some());
    }

    #[test]
    fn test_payment_partial_refund() {
        let mut payment = create_test_payment();
        
        // Process to success first
        payment.start_processing().unwrap();
        payment.mark_successful().unwrap();
        
        let refund_amount = Money::from_major_units(50.0, "USD".to_string()).unwrap();
        assert!(payment.refund(Some(&refund_amount), "Partial refund".to_string()).is_ok());
        
        // Check metadata contains refund amount
        let metadata = payment.metadata().unwrap();
        assert!(metadata.get("refund_amount_cents").is_some());
    }

    #[test]
    fn test_payment_refund_amount_validation() {
        let mut payment = create_test_payment();
        
        payment.start_processing().unwrap();
        payment.mark_successful().unwrap();
        
        // Cannot refund more than original amount
        let excessive_amount = Money::from_major_units(200.0, "USD".to_string()).unwrap();
        assert!(payment.refund(Some(&excessive_amount), "Too much".to_string()).is_err());
        
        // Cannot refund different currency
        let wrong_currency = Money::from_major_units(50.0, "JPY".to_string()).unwrap();
        assert!(payment.refund(Some(&wrong_currency), "Wrong currency".to_string()).is_err());
    }

    #[test]
    fn test_payment_cannot_modify_final() {
        let mut payment = create_test_payment();
        
        // Process to failed state
        payment.start_processing().unwrap();
        payment.mark_failed("Card declined".to_string()).unwrap();
        
        // Cannot modify description of final payment
        assert!(payment.update_description(Some("New desc".to_string())).is_err());
        
        // Cannot modify metadata (except internal_note)
        assert!(payment.update_metadata("key".to_string(), serde_json::Value::String("value".to_string())).is_err());
        
        // But can add internal notes
        assert!(payment.update_metadata("internal_note".to_string(), serde_json::Value::String("Note".to_string())).is_ok());
    }
}