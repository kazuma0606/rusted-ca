//domain/repository/account_command_repository.rs
// Account Command Repository トレイト
// 2025/8/28

use crate::domain::entity::account::Account;
use crate::domain::value_object::{AccountId, Email, Money};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait AccountCommandRepositoryInterface: Send + Sync {
    // 基本的なCRUD操作
    async fn save(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn update(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(
        &self,
        account_id: &AccountId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    // 残高操作（分離された操作として実装）
    async fn update_balance(
        &self,
        account_id: &AccountId,
        new_balance: &Money,
        updated_at: DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    // トランザクション的操作
    async fn save_batch(
        &self,
        accounts: &[Account],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    // 重複チェック用
    async fn exists_by_email(
        &self,
        email: &Email,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn exists_by_id(
        &self,
        account_id: &AccountId,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}