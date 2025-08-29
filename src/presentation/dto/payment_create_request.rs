//presentation/dto/payment_create_request.rs
// Payment Create Request DTO
// 2025/8/28

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymentCreateRequest {
    pub account_id: String,
    pub amount: f64,
    pub currency_code: String,
    pub payment_method: String, // "CARD", "BANK_TRANSFER", "DIGITAL_WALLET"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl PaymentCreateRequest {
    pub fn new(
        account_id: String,
        amount: f64,
        currency_code: String,
        payment_method: String,
    ) -> Self {
        Self {
            account_id,
            amount,
            currency_code,
            payment_method,
            description: None,
            customer_email: None,
            metadata: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_customer_email(mut self, customer_email: String) -> Self {
        self.customer_email = Some(customer_email);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_create_request_creation() {
        let request = PaymentCreateRequest::new(
            "account-123".to_string(),
            100.50,
            "USD".to_string(),
            "CARD".to_string(),
        )
        .with_description("Test payment".to_string())
        .with_customer_email("customer@example.com".to_string());

        assert_eq!(request.account_id, "account-123");
        assert_eq!(request.amount, 100.50);
        assert_eq!(request.currency_code, "USD");
        assert_eq!(request.payment_method, "CARD");
        assert_eq!(request.description, Some("Test payment".to_string()));
        assert_eq!(request.customer_email, Some("customer@example.com".to_string()));
    }

    #[test]
    fn test_payment_create_request_serialization() {
        let request = PaymentCreateRequest::new(
            "account-456".to_string(),
            75.25,
            "EUR".to_string(),
            "DIGITAL_WALLET".to_string(),
        );

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: PaymentCreateRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.account_id, deserialized.account_id);
        assert_eq!(request.amount, deserialized.amount);
        assert_eq!(request.currency_code, deserialized.currency_code);
        assert_eq!(request.payment_method, deserialized.payment_method);
    }
}