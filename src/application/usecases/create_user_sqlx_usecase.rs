use crate::domain::entity_sqlx::user_sqlx::UserSqlx;
use crate::presentation::dto::user_create_request_sqlx::UserCreateRequestSqlx;
use crate::presentation::dto::user_response_sqlx::UserResponseSqlx;
use crate::shared::error::application_error::ApplicationResult;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

#[async_trait]
pub trait CreateUserSqlxUsecaseInterface: Send + Sync {
    async fn create_user(&self, req: UserCreateRequestSqlx) -> ApplicationResult<UserResponseSqlx>;
}

pub struct CreateUserSqlxUsecase<R: Send + Sync> {
    pub repository: R,
}

#[async_trait]
impl<R> CreateUserSqlxUsecaseInterface for CreateUserSqlxUsecase<R>
where
    R: Send + Sync + CreateUserSqlxRepositoryInterface,
{
    async fn create_user(&self, req: UserCreateRequestSqlx) -> ApplicationResult<UserResponseSqlx> {
        let now = Utc::now().naive_utc();
        let user = UserSqlx {
            id: Uuid::new_v4().to_string(),
            email: req.email,
            name: req.name,
            password_hash: req.password, // 本番ではハッシュ化必須
            phone: req.phone,
            birth_date: req.birth_date,
            created_at: now,
            updated_at: now,
        };
        let saved = self.repository.save_user(&user).await?;
        Ok(UserResponseSqlx::from(saved))
    }
}

#[async_trait]
pub trait CreateUserSqlxRepositoryInterface: Send + Sync {
    async fn save_user(&self, user: &UserSqlx) -> ApplicationResult<UserSqlx>;
}
