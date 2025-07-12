use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxRepositoryInterface;
use crate::domain::entity_sqlx::user_sqlx::UserSqlx;
use crate::shared::error::application_error::ApplicationError;
use crate::shared::error::infrastructure_error::InfrastructureError;
use async_trait::async_trait;
use sqlx::MySqlPool;

pub struct TiDBUserSqlxRepository {
    pub pool: MySqlPool,
}

#[async_trait]
impl CreateUserSqlxRepositoryInterface for TiDBUserSqlxRepository {
    async fn save_user(&self, user: &UserSqlx) -> Result<UserSqlx, ApplicationError> {
        let query = r#"
            INSERT INTO users (id, email, name, password_hash, phone, birth_date, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        let result = sqlx::query(query)
            .bind(&user.id)
            .bind(&user.email)
            .bind(&user.name)
            .bind(&user.password_hash)
            .bind(&user.phone)
            .bind(&user.birth_date)
            .bind(&user.created_at)
            .bind(&user.updated_at)
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => Ok(user.clone()),
            Err(e) => Err(ApplicationError::Infrastructure(
                InfrastructureError::DatabaseQuery {
                    query: query.to_string(),
                    message: e.to_string(),
                },
            )),
        }
    }
}
