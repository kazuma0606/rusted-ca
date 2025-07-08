//domain/repository/user_command_repository.rs
// Command Repository トレイト
// 2025/7/8

use crate::domain::entity::user::User;
use async_trait::async_trait;

#[async_trait]
pub trait UserCommandRepositoryInterface: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn find_by_email_for_duplicate_check(
        &self,
        email: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}
