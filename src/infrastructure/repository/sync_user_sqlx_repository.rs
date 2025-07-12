use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxRepositoryInterface;
use crate::domain::entity_sqlx::user_sqlx::UserSqlx;
use crate::shared::error::application_error::ApplicationError;
use crate::shared::error::infrastructure_error::InfrastructureError;
use async_trait::async_trait;

pub struct SyncUserSqlxRepository<T, R>
where
    T: Send + Sync,
    R: Send + Sync,
{
    pub tidb: T,
    pub redis: R,
}

#[async_trait]
impl<T, R> CreateUserSqlxRepositoryInterface for SyncUserSqlxRepository<T, R>
where
    T: Send + Sync + CreateUserSqlxRepositoryInterface,
    R: Send + Sync + CreateUserSqlxRepositoryInterface,
{
    async fn save_user(&self, user: &UserSqlx) -> Result<UserSqlx, ApplicationError> {
        let saved = self.tidb.save_user(user).await?;
        let _ = self.redis.save_user(user).await; // Redis側のエラーは握りつぶす例
        Ok(saved)
    }
}
