use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxRepositoryInterface;
use crate::domain::entity_sqlx::user_sqlx::UserSqlx;
use crate::shared::error::application_error::ApplicationError;
use crate::shared::error::infrastructure_error::InfrastructureError;
use async_trait::async_trait;
use deadpool_redis::{Pool, redis::AsyncCommands};

pub struct RedisUserSqlxRepository {
    pub pool: Pool,
}

#[async_trait]
impl CreateUserSqlxRepositoryInterface for RedisUserSqlxRepository {
    async fn save_user(&self, user: &UserSqlx) -> Result<UserSqlx, ApplicationError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            ApplicationError::Infrastructure(InfrastructureError::ExternalService {
                service: "redis".to_string(),
                status: "connection_failed".to_string(),
                message: e.to_string(),
            })
        })?;

        let key = format!("user:{}", user.id);
        let birth_date = user.birth_date.map(|d| d.to_string()).unwrap_or_default();
        let phone = user.phone.clone().unwrap_or_default();
        let created_at = user.created_at.to_string();
        let updated_at = user.updated_at.to_string();

        let fields = [
            ("id", user.id.as_str()),
            ("email", user.email.as_str()),
            ("name", user.name.as_str()),
            ("password_hash", user.password_hash.as_str()),
            ("phone", phone.as_str()),
            ("birth_date", birth_date.as_str()),
            ("created_at", created_at.as_str()),
            ("updated_at", updated_at.as_str()),
        ];

        let result: Result<(), _> = conn.hset_multiple(&key, &fields).await;

        match result {
            Ok(_) => Ok(user.clone()),
            Err(e) => Err(ApplicationError::Infrastructure(
                InfrastructureError::ExternalService {
                    service: "redis".to_string(),
                    status: "hset_failed".to_string(),
                    message: e.to_string(),
                },
            )),
        }
    }
}
