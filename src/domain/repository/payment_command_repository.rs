//domain/repository/payment_command_repository.rs
// Payment Command Repository Interface (CQRS Write Side)
// 2025/8/28

use crate::domain::entity::payment::Payment;
use crate::domain::value_object::{PaymentId, ReferenceNumber};
use crate::shared::error::domain_error::DomainResult;
use async_trait::async_trait;

#[async_trait]
pub trait PaymentCommandRepository: Send + Sync {
    /// Create a new payment
    async fn create(&self, payment: &Payment) -> DomainResult<()>;

    /// Update an existing payment
    async fn update(&self, payment: &Payment) -> DomainResult<()>;

    /// Delete a payment (soft delete)
    async fn delete(&self, payment_id: &PaymentId) -> DomainResult<()>;

    /// Check if a payment exists by ID
    async fn exists_by_id(&self, payment_id: &PaymentId) -> DomainResult<bool>;

    /// Check if a reference number is unique
    async fn is_reference_unique(&self, reference_number: &ReferenceNumber) -> DomainResult<bool>;

    /// Begin transaction
    async fn begin_transaction(&self) -> DomainResult<()>;

    /// Commit transaction
    async fn commit_transaction(&self) -> DomainResult<()>;

    /// Rollback transaction
    async fn rollback_transaction(&self) -> DomainResult<()>;
}