//presentation/dto/account_update_request.rs
// Account Update Request DTO
// 2025/8/28

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merchant_name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, // "ACTIVE", "SUSPENDED", "CLOSED"
}

impl AccountUpdateRequest {
    pub fn new() -> Self {
        Self {
            merchant_name: None,
            status: None,
        }
    }

    pub fn with_merchant_name(mut self, merchant_name: String) -> Self {
        self.merchant_name = Some(merchant_name);
        self
    }

    pub fn with_status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.merchant_name.is_none() && self.status.is_none()
    }
}

impl Default for AccountUpdateRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_update_request_builder() {
        let request = AccountUpdateRequest::new()
            .with_merchant_name("Updated Merchant".to_string())
            .with_status("SUSPENDED".to_string());

        assert_eq!(request.merchant_name, Some("Updated Merchant".to_string()));
        assert_eq!(request.status, Some("SUSPENDED".to_string()));
        assert!(!request.is_empty());
    }

    #[test]
    fn test_account_update_request_empty() {
        let request = AccountUpdateRequest::new();
        assert!(request.is_empty());
    }

    #[test]
    fn test_account_update_request_serialization() {
        let request = AccountUpdateRequest::new()
            .with_merchant_name("Test Merchant".to_string());

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AccountUpdateRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.merchant_name, deserialized.merchant_name);
        assert_eq!(request.status, deserialized.status);
    }
}