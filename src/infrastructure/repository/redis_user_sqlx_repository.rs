use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxRepositoryInterface;
use crate::domain::entity_sqlx::user_sqlx::UserSqlx;
use crate::shared::error::application_error::ApplicationError;
use crate::shared::error::infrastructure_error::InfrastructureError;
use async_trait::async_trait;
use deadpool_redis::{Pool, redis::AsyncCommands};

#[derive(Clone)]
pub struct RedisUserSqlxRepository {
    pub pool: Pool,
}

use crate::shared::error::application_error::ApplicationResult;

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
    async fn get_user_by_id(&self, user_id: &str) -> Result<Option<UserSqlx>, ApplicationError> {
        println!("Called: Redis");
        self.get_user_by_id_impl(user_id).await
    }

    async fn update_user(&self, user: &UserSqlx) -> ApplicationResult<UserSqlx> {
        // Redisの場合は既存のsave_userを再利用（上書き）
        self.save_user(user).await
    }

    async fn delete_user(&self, user_id: &str) -> ApplicationResult<()> {
        let mut conn = self.pool.get().await.map_err(|e| {
            ApplicationError::Infrastructure(InfrastructureError::ExternalService {
                service: "redis".to_string(),
                status: "connection_failed".to_string(),
                message: e.to_string(),
            })
        })?;
        let key = format!("user:{}", user_id);
        let result: Result<(), _> = conn.del(&key).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(ApplicationError::Infrastructure(
                InfrastructureError::ExternalService {
                    service: "redis".to_string(),
                    status: "del_failed".to_string(),
                    message: e.to_string(),
                },
            )),
        }
    }
}

impl RedisUserSqlxRepository {
    pub async fn get_user_by_id_impl(
        &self,
        user_id: &str,
    ) -> Result<Option<UserSqlx>, ApplicationError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            ApplicationError::Infrastructure(InfrastructureError::ExternalService {
                service: "redis".to_string(),
                status: "connection_failed".to_string(),
                message: e.to_string(),
            })
        })?;
        let key = format!("user:{}", user_id);
        let map: std::collections::HashMap<String, String> =
            conn.hgetall(&key).await.map_err(|e| {
                ApplicationError::Infrastructure(InfrastructureError::ExternalService {
                    service: "redis".to_string(),
                    status: "hgetall_failed".to_string(),
                    message: e.to_string(),
                })
            })?;
        if map.is_empty() {
            return Ok(None);
        }
        // 必須フィールドの存在チェック
        let id = map.get("id").cloned().unwrap_or_default();
        let email = map.get("email").cloned().unwrap_or_default();
        let name = map.get("name").cloned().unwrap_or_default();
        let password_hash = map.get("password_hash").cloned().unwrap_or_default();
        let phone = map.get("phone").cloned().filter(|s| !s.is_empty());
        let birth_date = map
            .get("birth_date")
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
        let created_at = map
            .get("created_at")
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f").ok())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());
        let updated_at = map
            .get("updated_at")
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f").ok())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());
        Ok(Some(UserSqlx {
            id,
            email,
            name,
            password_hash,
            phone,
            birth_date,
            created_at,
            updated_at,
        }))
    }

    async fn update_user(&self, user: &UserSqlx) -> ApplicationResult<UserSqlx> {
        // Redisの場合は既存のsave_userを再利用（上書き）
        self.save_user(user).await
    }
}
