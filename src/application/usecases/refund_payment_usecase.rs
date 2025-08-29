//application/usecases/refund_payment_usecase.rs
// Refund Payment Use Case
// 2025/8/28

use crate::domain::entity::payment::Payment;
use crate::domain::repository::payment_command_repository::PaymentCommandRepository;
use crate::domain::repository::payment_query_repository::PaymentQueryRepository;
use crate::domain::value_object::{Money, PaymentId};
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use std::sync::Arc;

pub struct RefundPaymentUseCase {
    command_repository: Arc<dyn PaymentCommandRepository>,
    query_repository: Arc<dyn PaymentQueryRepository>,
}

impl RefundPaymentUseCase {
    pub fn new(
        command_repository: Arc<dyn PaymentCommandRepository>,
        query_repository: Arc<dyn PaymentQueryRepository>,
    ) -> Self {
        Self {
            command_repository,
            query_repository,
        }
    }

    /// Process full refund
    pub async fn full_refund(
        &self,
        payment_id: PaymentId,
        reason: String,
    ) -> ApplicationResult<Payment> {
        // Find the payment
        let mut payment = self
            .query_repository
            .find_by_id(&payment_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::NotFound {
                resource: "Payment".to_string(),
                id: payment_id.value().to_string(),
            })?;

        // Check if payment can be refunded
        if !payment.can_be_refunded() {
            return Err(ApplicationError::Domain(
                crate::shared::error::domain_error::DomainError::BusinessRuleViolation {
                    rule: "Refund".to_string(),
                    message: format!(
                        "Payment with status '{}' cannot be refunded",
                        payment.status().as_str()
                    ),
                },
            ));
        }

        // Process full refund
        payment
            .refund(None, reason)
            .map_err(|e| ApplicationError::Domain(e))?;

        // Save updated payment
        self.command_repository
            .update(&payment)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(payment)
    }

    /// Process partial refund
    pub async fn partial_refund(
        &self,
        payment_id: PaymentId,
        refund_amount: Money,
        reason: String,
    ) -> ApplicationResult<Payment> {
        // Find the payment
        let mut payment = self
            .query_repository
            .find_by_id(&payment_id)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ApplicationError::NotFound {
                resource: "Payment".to_string(),
                id: payment_id.value().to_string(),
            })?;

        // Check if payment can be refunded
        if !payment.can_be_refunded() {
            return Err(ApplicationError::Domain(
                crate::shared::error::domain_error::DomainError::BusinessRuleViolation {
                    rule: "Refund".to_string(),
                    message: format!(
                        "Payment with status '{}' cannot be refunded",
                        payment.status().as_str()
                    ),
                },
            ));
        }

        // Validate refund amount
        if refund_amount.currency_code() != payment.amount().currency_code() {
            return Err(ApplicationError::ValidationError(format!(
                "Refund currency '{}' does not match payment currency '{}'",
                refund_amount.currency_code(),
                payment.amount().currency_code()
            )));
        }

        if &refund_amount > payment.amount() {
            return Err(ApplicationError::ValidationError(
                "Refund amount cannot exceed original payment amount".to_string(),
            ));
        }

        if !refund_amount.is_positive() {
            return Err(ApplicationError::ValidationError(
                "Refund amount must be positive".to_string(),
            ));
        }

        // Process partial refund
        payment
            .refund(Some(&refund_amount), reason)
            .map_err(|e| ApplicationError::Domain(e))?;

        // Save updated payment
        self.command_repository
            .update(&payment)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(payment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::usecases::create_payment_usecase::tests::MockPaymentRepository;
    use crate::domain::value_object::{AccountId, Money, PaymentMethod};

    #[tokio::test]
    async fn test_full_refund_success() {
        let repository = Arc::new(MockPaymentRepository::new());
        let usecase = RefundPaymentUseCase::new(repository.clone(), repository.clone());

        // Create a successful payment
        let account_id = AccountId::generate();
        let amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        let mut payment = Payment::create_new(
            account_id,
            amount,
            PaymentMethod::Card,
            Some("Test payment".to_string()),
            None,
            None,
        )
        .unwrap();

        // Process payment through correct state transitions
        payment.start_processing().unwrap();
        payment.mark_successful().unwrap();

        // Add to mock repository
        repository.create(&payment).await.unwrap();

        // Test full refund
        let result = usecase
            .full_refund(payment.id().clone(), "Customer request".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_partial_refund_success() {
        let repository = Arc::new(MockPaymentRepository::new());
        let usecase = RefundPaymentUseCase::new(repository.clone(), repository.clone());

        // Create a successful payment
        let account_id = AccountId::generate();
        let amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        let mut payment = Payment::create_new(
            account_id,
            amount,
            PaymentMethod::Card,
            Some("Test payment".to_string()),
            None,
            None,
        )
        .unwrap();

        // Process payment through correct state transitions
        payment.start_processing().unwrap();
        payment.mark_successful().unwrap();

        // Add to mock repository
        repository.create(&payment).await.unwrap();

        // Test partial refund
        let refund_amount = Money::from_major_units(50.0, "USD".to_string()).unwrap();
        let result = usecase
            .partial_refund(
                payment.id().clone(),
                refund_amount,
                "Partial refund".to_string(),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_refund_fails_for_pending_payment() {
        let repository = Arc::new(MockPaymentRepository::new());
        let usecase = RefundPaymentUseCase::new(repository.clone(), repository.clone());

        // Create a pending payment
        let account_id = AccountId::generate();
        let amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        let payment = Payment::create_new(
            account_id,
            amount,
            PaymentMethod::Card,
            Some("Test payment".to_string()),
            None,
            None,
        )
        .unwrap();

        // Add to mock repository
        repository.create(&payment).await.unwrap();

        // Test refund should fail
        let result = usecase
            .full_refund(payment.id().clone(), "Customer request".to_string())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_partial_refund_amount_validation() {
        let repository = Arc::new(MockPaymentRepository::new());
        let usecase = RefundPaymentUseCase::new(repository.clone(), repository.clone());

        // Create a successful payment
        let account_id = AccountId::generate();
        let amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        let mut payment = Payment::create_new(
            account_id,
            amount,
            PaymentMethod::Card,
            Some("Test payment".to_string()),
            None,
            None,
        )
        .unwrap();

        // Process payment through correct state transitions
        payment.start_processing().unwrap();
        payment.mark_successful().unwrap();

        // Add to mock repository
        repository.create(&payment).await.unwrap();

        // Test refund with amount exceeding payment amount
        let refund_amount = Money::from_major_units(150.0, "USD".to_string()).unwrap();
        let result = usecase
            .partial_refund(
                payment.id().clone(),
                refund_amount,
                "Excessive refund".to_string(),
            )
            .await;
        assert!(result.is_err());
    }
}
