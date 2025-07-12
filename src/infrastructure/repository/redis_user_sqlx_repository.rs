use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxRepositoryInterface;
use crate::domain::entity_sqlx::user_sqlx::UserSqlx;
use crate::shared::error::application_error::ApplicationError;
use crate::shared::error::infrastructure_error::InfrastructureError;
use async_trait::async_trait;
use deadpool_redis::{Pool, redis::AsyncCommands};
use serde_json;

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
        let value = serde_json::to_string(user).map_err(|e| {
            ApplicationError::Infrastructure(InfrastructureError::DataSerialization {
                data_type: "UserSqlx".to_string(),
                message: e.to_string(),
            })
        })?;

        let result: Result<(), _> = conn.set(key, value).await;

        match result {
            Ok(_) => Ok(user.clone()),
            Err(e) => Err(ApplicationError::Infrastructure(
                InfrastructureError::ExternalService {
                    service: "redis".to_string(),
                    status: "set_failed".to_string(),
                    message: e.to_string(),
                },
            )),
        }
    }
}
