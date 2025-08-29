//presentation/dto/payment_operation_request.rs
// Payment Operation Request DTO (Process, Cancel, Refund)
// 2025/8/28

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentOperationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>, // For partial refunds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>, // For partial refunds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl PaymentOperationRequest {
    pub fn new() -> Self {
        Self {
            reason: None,
            amount: None,
            currency_code: None,
            metadata: None,
        }
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn with_refund_amount(mut self, amount: f64, currency_code: String) -> Self {
        self.amount = Some(amount);
        self.currency_code = Some(currency_code);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn is_partial_refund(&self) -> bool {
        self.amount.is_some() && self.currency_code.is_some()
    }
}

impl Default for PaymentOperationRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentSearchQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_after: Option<String>, // ISO 8601 datetime
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_before: Option<String>, // ISO 8601 datetime
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
}

impl PaymentSearchQuery {
    pub fn new() -> Self {
        Self {
            account_id: None,
            status: None,
            customer_email: None,
            currency_code: None,
            min_amount: None,
            max_amount: None,
            created_after: None,
            created_before: None,
            limit: Some(50), // Default limit
            offset: Some(0),
        }
    }
}

impl Default for PaymentSearchQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_operation_request_creation() {
        let request = PaymentOperationRequest::new()
            .with_reason("Customer request".to_string())
            .with_refund_amount(50.0, "USD".to_string());

        assert_eq!(request.reason, Some("Customer request".to_string()));
        assert_eq!(request.amount, Some(50.0));
        assert_eq!(request.currency_code, Some("USD".to_string()));
        assert!(request.is_partial_refund());
    }

    #[test]
    fn test_payment_operation_request_full_refund() {
        let request = PaymentOperationRequest::new()
            .with_reason("Defective product".to_string());

        assert_eq!(request.reason, Some("Defective product".to_string()));
        assert!(!request.is_partial_refund());
    }

    #[test]
    fn test_payment_search_query_defaults() {
        let query = PaymentSearchQuery::default();
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.offset, Some(0));
        assert!(query.account_id.is_none());
    }

    #[test]
    fn test_payment_operation_request_serialization() {
        let request = PaymentOperationRequest::new()
            .with_reason("Test cancellation".to_string());

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: PaymentOperationRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.reason, deserialized.reason);
        assert_eq!(request.amount, deserialized.amount);
    }
}