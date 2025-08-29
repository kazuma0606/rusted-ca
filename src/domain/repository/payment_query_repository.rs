//domain/repository/payment_query_repository.rs
// Payment Query Repository Interface (CQRS Read Side)
// 2025/8/28

use crate::domain::entity::payment::Payment;
use crate::domain::value_object::{AccountId, PaymentId, PaymentStatus, ReferenceNumber};
use crate::shared::error::domain_error::DomainResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct PaymentSearchCriteria {
    pub account_id: Option<AccountId>,
    pub status: Option<PaymentStatus>,
    pub customer_email: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub min_amount_cents: Option<i64>,
    pub max_amount_cents: Option<i64>,
    pub currency_code: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl Default for PaymentSearchCriteria {
    fn default() -> Self {
        Self {
            account_id: None,
            status: None,
            customer_email: None,
            created_after: None,
            created_before: None,
            min_amount_cents: None,
            max_amount_cents: None,
            currency_code: None,
            limit: Some(50), // Default limit
            offset: Some(0),
        }
    }
}

#[async_trait]
pub trait PaymentQueryRepository: Send + Sync {
    /// Find payment by ID
    async fn find_by_id(&self, payment_id: &PaymentId) -> DomainResult<Option<Payment>>;

    /// Find payment by reference number
    async fn find_by_reference(&self, reference_number: &ReferenceNumber) -> DomainResult<Option<Payment>>;

    /// Find payments by account ID
    async fn find_by_account_id(&self, account_id: &AccountId) -> DomainResult<Vec<Payment>>;

    /// Find payments by status
    async fn find_by_status(&self, status: &PaymentStatus) -> DomainResult<Vec<Payment>>;

    /// Search payments with criteria
    async fn search(&self, criteria: &PaymentSearchCriteria) -> DomainResult<Vec<Payment>>;

    /// Count payments by criteria
    async fn count(&self, criteria: &PaymentSearchCriteria) -> DomainResult<u64>;

    /// Find payments requiring processing (pending/stuck payments)
    async fn find_requiring_processing(&self, older_than_minutes: u32) -> DomainResult<Vec<Payment>>;

    /// Find successful payments for account within date range
    async fn find_successful_payments(
        &self,
        account_id: &AccountId,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> DomainResult<Vec<Payment>>;

    /// Get payment statistics for account
    async fn get_payment_stats(&self, account_id: &AccountId) -> DomainResult<PaymentStats>;
}

#[derive(Debug, Clone)]
pub struct PaymentStats {
    pub total_payments: u64,
    pub successful_payments: u64,
    pub failed_payments: u64,
    pub pending_payments: u64,
    pub total_amount_cents: i64,
    pub successful_amount_cents: i64,
    pub average_amount_cents: i64,
    pub currency_code: String,
}