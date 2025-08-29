// src/shared/domain_error.rs

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DomainError {
    // Value Object Validation Errors
    #[error("Invalid email format: '{email}' - {reason}")]
    InvalidEmail { email: String, reason: String },

    #[error("Invalid user name: '{name}' - {reason}")]
    InvalidUserName { name: String, reason: String },

    #[error("Invalid password: {reason}")]
    InvalidPassword { reason: String },

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    // Financial Value Object Errors
    #[error("Invalid money: {reason}")]
    InvalidMoney { reason: String },

    #[error("Currency mismatch: expected '{expected}', got '{actual}'")]
    CurrencyMismatch { expected: String, actual: String },

    // Business Rule Violations
    #[error("Business rule violation: {rule} - {message}")]
    BusinessRuleViolation { rule: String, message: String },

    #[error("Entity validation failed: {entity} - {field}: {message}")]
    EntityValidationFailed {
        entity: String,
        field: String,
        message: String,
    },

    // Payment-related Errors
    #[error("Invalid payment ID: {reason}")]
    InvalidPaymentId { reason: String },

    #[error("Invalid payment method: {reason}")]
    InvalidPaymentMethod { reason: String },

    #[error("Invalid payment status: {reason}")]
    InvalidPaymentStatus { reason: String },

    #[error("Invalid reference number: {reason}")]
    InvalidReferenceNumber { reason: String },

    // Domain Logic Errors
    #[error("Invariant violation: {message}")]
    InvariantViolation { message: String },
}

// Domain Layer Result Type
pub type DomainResult<T> = Result<T, DomainError>;
