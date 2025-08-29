//presentation/dto/account_create_request.rs
// Account Create Request DTO
// 2025/8/28

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountCreateRequest {
    pub merchant_name: String,
    pub email: String,
    pub currency_code: String,
}

impl AccountCreateRequest {
    pub fn new(merchant_name: String, email: String, currency_code: String) -> Self {
        Self {
            merchant_name,
            email,
            currency_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_create_request_serialization() {
        let request = AccountCreateRequest::new(
            "Test Merchant".to_string(),
            "test@merchant.com".to_string(),
            "USD".to_string(),
        );

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AccountCreateRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.merchant_name, deserialized.merchant_name);
        assert_eq!(request.email, deserialized.email);
        assert_eq!(request.currency_code, deserialized.currency_code);
    }
}