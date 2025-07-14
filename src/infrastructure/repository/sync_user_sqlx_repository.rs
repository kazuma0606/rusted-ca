use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxRepositoryInterface;
use crate::domain::entity_sqlx::user_sqlx::UserSqlx;
use crate::shared::error::application_error::ApplicationError;
use crate::shared::error::infrastructure_error::InfrastructureError;
use async_trait::async_trait;

#[derive(Clone)]
pub struct SyncUserSqlxRepository<T, R>
where
    T: Send + Sync + Clone,
    R: Send + Sync + Clone,
{
    pub tidb: T,
    pub redis: R,
}

#[async_trait]
impl<T, R> CreateUserSqlxRepositoryInterface for SyncUserSqlxRepository<T, R>
where
    T: Send + Sync + CreateUserSqlxRepositoryInterface + Clone,
    R: Send + Sync + CreateUserSqlxRepositoryInterface + Clone,
{
    async fn save_user(&self, user: &UserSqlx) -> Result<UserSqlx, ApplicationError> {
        let saved = self.tidb.save_user(user).await?;
        let _ = self.redis.save_user(user).await; // Redis側のエラーは握りつぶす例
        Ok(saved)
    }

    async fn get_user_by_id(&self, user_id: &str) -> Result<Option<UserSqlx>, ApplicationError> {
        // 1. Redisから取得
        if let Some(user) = self.redis.get_user_by_id(user_id).await? {
            return Ok(Some(user));
        }
        // 2. TiDBから取得
        if let Some(user) = self.tidb.get_user_by_id(user_id).await? {
            // 3. Redisにキャッシュ
            let _ = self.redis.save_user(&user).await;
            return Ok(Some(user));
        }
        Ok(None)
    }

    async fn update_user(&self, user: &UserSqlx) -> Result<UserSqlx, ApplicationError> {
        // 1. MySQLでUpdate
        let updated = self.tidb.update_user(user).await?;
        // 2. Redisを更新（キャッシュ更新）
        let _ = self.redis.update_user(user).await; // Redis側のエラーは握りつぶす例
        Ok(updated)
    }
}
