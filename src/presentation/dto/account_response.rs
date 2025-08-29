//presentation/dto/account_response.rs
// Account Response DTO
// 2025/8/28

use crate::domain::entity::account::Account;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountResponse {
    pub id: String,
    pub merchant_name: String,
    pub email: String,
    pub status: String,
    pub balance: BalanceResponse,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceResponse {
    pub amount: f64,
    pub currency_code: String,
    pub amount_cents: i64,
}

impl AccountResponse {
    pub fn from_account(account: &Account) -> Self {
        Self {
            id: account.id().value().to_string(),
            merchant_name: account.merchant_name().value().to_string(),
            email: account.email().value().to_string(),
            status: account.status().as_str().to_string(),
            balance: BalanceResponse {
                amount: account.balance().to_major_units(),
                currency_code: account.balance().currency_code().to_string(),
                amount_cents: account.balance().amount_cents(),
            },
            created_at: *account.created_at(),
            updated_at: *account.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountListResponse {
    pub accounts: Vec<AccountResponse>,
    pub total_count: u64,
    pub page: u64,
    pub page_size: u64,
}

impl AccountListResponse {
    pub fn new(accounts: Vec<AccountResponse>, total_count: u64, page: u64, page_size: u64) -> Self {
        Self {
            accounts,
            total_count,
            page,
            page_size,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceOnlyResponse {
    pub balance: BalanceResponse,
}

impl BalanceOnlyResponse {
    pub fn from_account(account: &Account) -> Self {
        Self {
            balance: BalanceResponse {
                amount: account.balance().to_major_units(),
                currency_code: account.balance().currency_code().to_string(),
                amount_cents: account.balance().amount_cents(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::account::Account;
    use crate::domain::value_object::{AccountId, AccountStatus, Email, MerchantName, Money};

    fn create_test_account() -> Account {
        let merchant_name = MerchantName::new("Test Merchant".to_string()).unwrap();
        let email = Email::new("test@merchant.com".to_string()).unwrap();
        Account::create_new(merchant_name, email, "USD".to_string()).unwrap()
    }

    #[test]
    fn test_account_response_from_account() {
        let account = create_test_account();
        let response = AccountResponse::from_account(&account);

        assert_eq!(response.merchant_name, "Test Merchant");
        assert_eq!(response.email, "test@merchant.com");
        assert_eq!(response.status, "ACTIVE");
        assert_eq!(response.balance.currency_code, "USD");
        assert_eq!(response.balance.amount, 0.0);
        assert_eq!(response.balance.amount_cents, 0);
    }

    #[test]
    fn test_balance_only_response_from_account() {
        let account = create_test_account();
        let response = BalanceOnlyResponse::from_account(&account);

        assert_eq!(response.balance.currency_code, "USD");
        assert_eq!(response.balance.amount, 0.0);
        assert_eq!(response.balance.amount_cents, 0);
    }

    #[test]
    fn test_account_list_response() {
        let account = create_test_account();
        let account_responses = vec![AccountResponse::from_account(&account)];
        let list_response = AccountListResponse::new(account_responses, 1, 1, 10);

        assert_eq!(list_response.accounts.len(), 1);
        assert_eq!(list_response.total_count, 1);
        assert_eq!(list_response.page, 1);
        assert_eq!(list_response.page_size, 10);
    }

    #[test]
    fn test_account_response_serialization() {
        let account = create_test_account();
        let response = AccountResponse::from_account(&account);

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AccountResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.merchant_name, deserialized.merchant_name);
        assert_eq!(response.email, deserialized.email);
        assert_eq!(response.status, deserialized.status);
        assert_eq!(response.balance.currency_code, deserialized.balance.currency_code);
    }
}