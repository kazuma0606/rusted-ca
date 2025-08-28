//application/dto/account_request_dto.rs
// Account関連のリクエストDTO
// 2025/8/28

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequestDto {
    pub merchant_name: String,
    pub email: String,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequestDto {
    pub merchant_name: Option<String>,
    // Note: email updates might require special verification in future
    // pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountCreditRequestDto {
    pub amount_cents: i64,
    pub currency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDebitRequestDto {
    pub amount_cents: i64,
    pub currency_code: String,
}