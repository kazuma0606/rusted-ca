//application/dto/account_response_dto.rs
// Account関連のレスポンスDTO
// 2025/8/28

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponseDto {
    pub id: String,
    pub merchant_name: String,
    pub email: String,
    pub status: String,
    pub balance_cents: i64,
    pub currency_code: String,
    pub balance_formatted: String, // Human-readable balance (e.g., "$12.50")
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Domain Account からResponseDTOへの変換
impl AccountResponseDto {
    pub fn from_domain(account: &crate::domain::entity::account::Account) -> Self {
        Self {
            id: account.id().value().to_string(),
            merchant_name: account.merchant_name().value().to_string(),
            email: account.email().value().to_string(),
            status: account.status().as_str().to_string(),
            balance_cents: account.balance().amount_cents(),
            currency_code: account.balance().currency_code().to_string(),
            balance_formatted: account.balance().to_string(),
            created_at: *account.created_at(),
            updated_at: *account.updated_at(),
        }
    }
}