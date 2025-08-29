//application/usecases/process_payment_usecase.rs
// Process Payment Use Case
// 2025/8/28

use crate::domain::entity::payment::Payment;
use crate::domain::repository::payment_command_repository::PaymentCommandRepository;
use crate::domain::repository::payment_query_repository::PaymentQueryRepository;
use crate::domain::value_object::PaymentId;
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use std::sync::Arc;

pub struct ProcessPaymentUseCase {
    command_repository: Arc<dyn PaymentCommandRepository>,
    query_repository: Arc<dyn PaymentQueryRepository>,
}

impl ProcessPaymentUseCase {
    pub fn new(
        command_repository: Arc<dyn PaymentCommandRepository>,
        query_repository: Arc<dyn PaymentQueryRepository>,
    ) -> Self {
        Self {
            command_repository,
            query_repository,
        }
    }

    /// Start payment processing
    pub async fn start_processing(&self, payment_id: PaymentId) -> ApplicationResult<Payment> {
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

        // Start processing
        payment
            .start_processing()
            .map_err(|e| ApplicationError::Domain(e))?;

        // Save updated payment
        self.command_repository
            .update(&payment)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(payment)
    }

    /// Mark payment as successful
    pub async fn mark_successful(&self, payment_id: PaymentId) -> ApplicationResult<Payment> {
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

        // Mark as successful
        payment
            .mark_successful()
            .map_err(|e| ApplicationError::Domain(e))?;

        // Save updated payment
        self.command_repository
            .update(&payment)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(payment)
    }

    /// Mark payment as failed
    pub async fn mark_failed(
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

        // Mark as failed
        payment
            .mark_failed(reason)
            .map_err(|e| ApplicationError::Domain(e))?;

        // Save updated payment
        self.command_repository
            .update(&payment)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(payment)
    }

    /// Process payment end-to-end (simulate external payment processing)
    pub async fn process_payment(&self, payment_id: PaymentId) -> ApplicationResult<Payment> {
        // Start processing
        let mut payment = self.start_processing(payment_id.clone()).await?;

        // Simulate external payment gateway processing
        let processing_result = self.simulate_payment_gateway(&payment).await;

        match processing_result {
            Ok(_) => {
                // Payment successful - update the payment object directly
                payment
                    .mark_successful()
                    .map_err(|e| ApplicationError::Domain(e))?;

                // Save updated payment
                self.command_repository
                    .update(&payment)
                    .await
                    .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

                Ok(payment)
            }
            Err(failure_reason) => {
                // Payment failed
                self.mark_failed(payment_id, failure_reason).await
            }
        }
    }

    /// Simulate external payment gateway (for testing/demo purposes)
    async fn simulate_payment_gateway(&self, payment: &Payment) -> Result<(), String> {
        // Simulate processing delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Simple simulation logic
        match payment.payment_method() {
            crate::domain::value_object::PaymentMethod::Card => {
                // 90% success rate for cards
                if payment.amount().amount_cents() > 1000000 {
                    // > $10,000
                    Err("Amount exceeds card limit".to_string())
                } else if payment.amount().amount_cents() <= 0 {
                    Err("Invalid amount".to_string())
                } else {
                    // Simulate occasional failures
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    if rng.gen_range(0..100) < 10 {
                        Err("Card declined".to_string())
                    } else {
                        Ok(())
                    }
                }
            }
            crate::domain::value_object::PaymentMethod::BankTransfer => {
                // Bank transfers are slower but more reliable
                if payment.amount().amount_cents() > 5000000 {
                    // > $50,000
                    Err("Amount exceeds daily transfer limit".to_string())
                } else {
                    Ok(())
                }
            }
            crate::domain::value_object::PaymentMethod::DigitalWallet => {
                // Digital wallets are fast and reliable
                if payment.amount().amount_cents() > 100000 {
                    // > $1,000
                    Err("Digital wallet daily limit exceeded".to_string())
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::usecases::create_payment_usecase::tests::MockPaymentRepository;
    use crate::domain::value_object::{AccountId, Money, PaymentMethod};

    #[tokio::test]
    async fn test_start_processing_success() {
        let repository = Arc::new(MockPaymentRepository::new());
        let usecase = ProcessPaymentUseCase::new(repository.clone(), repository.clone());

        // First create a payment
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

        // Test start processing
        let result = usecase.start_processing(payment.id().clone()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_payment_simulation() {
        let repository = Arc::new(MockPaymentRepository::new());
        let usecase = ProcessPaymentUseCase::new(repository.clone(), repository.clone());

        // Create a small payment (should succeed)
        let account_id = AccountId::generate();
        let amount = Money::from_major_units(50.0, "USD".to_string()).unwrap();
        let payment = Payment::create_new(
            account_id,
            amount,
            PaymentMethod::Card,
            Some("Small test payment".to_string()),
            None,
            None,
        )
        .unwrap();

        repository.create(&payment).await.unwrap();

        let result = usecase.process_payment(payment.id().clone()).await;
        if let Err(ref e) = result {
            println!("Process payment failed: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Process payment should succeed: {:?}",
            result
        );
    }
}
