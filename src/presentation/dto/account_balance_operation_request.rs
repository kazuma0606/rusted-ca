//presentation/dto/account_balance_operation_request.rs
// Account Balance Operation Request DTO (Credit/Debit)
// 2025/8/28

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountBalanceOperationRequest {
    pub amount: f64,
    pub currency_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

impl AccountBalanceOperationRequest {
    pub fn new(amount: f64, currency_code: String) -> Self {
        Self {
            amount,
            currency_code,
            description: None,
            reference: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_reference(mut self, reference: String) -> Self {
        self.reference = Some(reference);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_operation_request_creation() {
        let request = AccountBalanceOperationRequest::new(100.50, "USD".to_string())
            .with_description("Test credit".to_string())
            .with_reference("REF-123".to_string());

        assert_eq!(request.amount, 100.50);
        assert_eq!(request.currency_code, "USD");
        assert_eq!(request.description, Some("Test credit".to_string()));
        assert_eq!(request.reference, Some("REF-123".to_string()));
    }

    #[test]
    fn test_balance_operation_request_serialization() {
        let request = AccountBalanceOperationRequest::new(50.25, "EUR".to_string());

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AccountBalanceOperationRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.amount, deserialized.amount);
        assert_eq!(request.currency_code, deserialized.currency_code);
        assert_eq!(request.description, deserialized.description);
        assert_eq!(request.reference, deserialized.reference);
    }
}