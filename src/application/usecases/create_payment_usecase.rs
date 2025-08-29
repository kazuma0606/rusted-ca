//application/usecases/create_payment_usecase.rs
// Create Payment Use Case
// 2025/8/28

use crate::domain::entity::payment::Payment;
use crate::domain::repository::payment_command_repository::PaymentCommandRepository;
use crate::domain::repository::payment_query_repository::PaymentQueryRepository;
use crate::domain::value_object::{AccountId, Money, PaymentMethod};
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use std::sync::Arc;

pub struct CreatePaymentUseCase {
    command_repository: Arc<dyn PaymentCommandRepository>,
    query_repository: Arc<dyn PaymentQueryRepository>,
}

impl CreatePaymentUseCase {
    pub fn new(
        command_repository: Arc<dyn PaymentCommandRepository>,
        query_repository: Arc<dyn PaymentQueryRepository>,
    ) -> Self {
        Self {
            command_repository,
            query_repository,
        }
    }

    pub async fn execute(
        &self,
        account_id: AccountId,
        amount: Money,
        payment_method: PaymentMethod,
        description: Option<String>,
        customer_email: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> ApplicationResult<Payment> {
        // Create new payment entity
        let payment = Payment::create_new(
            account_id,
            amount,
            payment_method,
            description,
            customer_email,
            metadata,
        )
        .map_err(|e| ApplicationError::Domain(e))?;

        // Ensure reference number is unique
        let is_unique = self
            .query_repository
            .find_by_reference(payment.reference_number())
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?
            .is_none();

        if !is_unique {
            return Err(ApplicationError::ValidationError(
                "Reference number already exists".to_string(),
            ));
        }

        // Save the payment
        self.command_repository
            .create(&payment)
            .await
            .map_err(|e| ApplicationError::RepositoryError(e.to_string()))?;

        Ok(payment)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::domain::value_object::PaymentId;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repository for testing
    pub struct MockPaymentRepository {
        payments: Arc<Mutex<HashMap<String, Payment>>>,
    }

    impl MockPaymentRepository {
        pub fn new() -> Self {
            Self {
                payments: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl PaymentCommandRepository for MockPaymentRepository {
        async fn create(
            &self,
            payment: &Payment,
        ) -> crate::shared::error::domain_error::DomainResult<()> {
            let mut payments = self.payments.lock().unwrap();
            payments.insert(payment.id().value().to_string(), payment.clone());
            Ok(())
        }

        async fn update(
            &self,
            _payment: &Payment,
        ) -> crate::shared::error::domain_error::DomainResult<()> {
            Ok(())
        }

        async fn delete(
            &self,
            _payment_id: &PaymentId,
        ) -> crate::shared::error::domain_error::DomainResult<()> {
            Ok(())
        }

        async fn exists_by_id(
            &self,
            _payment_id: &PaymentId,
        ) -> crate::shared::error::domain_error::DomainResult<bool> {
            Ok(false)
        }

        async fn is_reference_unique(
            &self,
            _reference_number: &crate::domain::value_object::ReferenceNumber,
        ) -> crate::shared::error::domain_error::DomainResult<bool> {
            Ok(true)
        }

        async fn begin_transaction(&self) -> crate::shared::error::domain_error::DomainResult<()> {
            Ok(())
        }

        async fn commit_transaction(&self) -> crate::shared::error::domain_error::DomainResult<()> {
            Ok(())
        }

        async fn rollback_transaction(
            &self,
        ) -> crate::shared::error::domain_error::DomainResult<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl PaymentQueryRepository for MockPaymentRepository {
        async fn find_by_id(
            &self,
            payment_id: &PaymentId,
        ) -> crate::shared::error::domain_error::DomainResult<Option<Payment>> {
            let payments = self.payments.lock().unwrap();
            Ok(payments.get(payment_id.value()).cloned())
        }

        async fn find_by_reference(
            &self,
            _reference_number: &crate::domain::value_object::ReferenceNumber,
        ) -> crate::shared::error::domain_error::DomainResult<Option<Payment>> {
            Ok(None) // Always unique for tests
        }

        async fn find_by_account_id(
            &self,
            _account_id: &AccountId,
        ) -> crate::shared::error::domain_error::DomainResult<Vec<Payment>> {
            Ok(vec![])
        }

        async fn find_by_status(
            &self,
            _status: &crate::domain::value_object::PaymentStatus,
        ) -> crate::shared::error::domain_error::DomainResult<Vec<Payment>> {
            Ok(vec![])
        }

        async fn search(
            &self,
            _criteria: &crate::domain::repository::payment_query_repository::PaymentSearchCriteria,
        ) -> crate::shared::error::domain_error::DomainResult<Vec<Payment>> {
            Ok(vec![])
        }

        async fn count(
            &self,
            _criteria: &crate::domain::repository::payment_query_repository::PaymentSearchCriteria,
        ) -> crate::shared::error::domain_error::DomainResult<u64> {
            Ok(0)
        }

        async fn find_requiring_processing(
            &self,
            _older_than_minutes: u32,
        ) -> crate::shared::error::domain_error::DomainResult<Vec<Payment>> {
            Ok(vec![])
        }

        async fn find_successful_payments(
            &self,
            _account_id: &AccountId,
            _from_date: chrono::DateTime<chrono::Utc>,
            _to_date: chrono::DateTime<chrono::Utc>,
        ) -> crate::shared::error::domain_error::DomainResult<Vec<Payment>> {
            Ok(vec![])
        }

        async fn get_payment_stats(
            &self,
            _account_id: &AccountId,
        ) -> crate::shared::error::domain_error::DomainResult<
            crate::domain::repository::payment_query_repository::PaymentStats,
        > {
            use crate::domain::repository::payment_query_repository::PaymentStats;
            Ok(PaymentStats {
                total_payments: 0,
                successful_payments: 0,
                failed_payments: 0,
                pending_payments: 0,
                total_amount_cents: 0,
                successful_amount_cents: 0,
                average_amount_cents: 0,
                currency_code: "USD".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn test_create_payment_success() {
        let repository = Arc::new(MockPaymentRepository::new());
        let usecase = CreatePaymentUseCase::new(repository.clone(), repository.clone());

        let account_id = AccountId::generate();
        let amount = Money::from_major_units(100.0, "USD".to_string()).unwrap();
        let payment_method = PaymentMethod::Card;

        let result = usecase
            .execute(
                account_id,
                amount,
                payment_method,
                Some("Test payment".to_string()),
                Some("customer@example.com".to_string()),
                None,
            )
            .await;

        assert!(result.is_ok());
        let payment = result.unwrap();
        assert!(payment.amount().is_positive());
        assert_eq!(payment.payment_method(), &PaymentMethod::Card);
        assert_eq!(payment.description(), Some(&"Test payment".to_string()));
    }
}
