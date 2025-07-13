use crate::domain::entity_sqlx::user_sqlx::UserSqlx;
use crate::presentation::dto::user_create_request_sqlx::UserCreateRequestSqlx;
use crate::presentation::dto::user_response_sqlx::UserResponseSqlx;
use crate::shared::error::application_error::ApplicationResult;
use crate::shared::error::infrastructure_error::PasswordHasherResult;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

#[async_trait]
pub trait CreateUserSqlxUsecaseInterface: Send + Sync {
    async fn create_user(&self, req: UserCreateRequestSqlx) -> ApplicationResult<UserResponseSqlx>;
}

#[async_trait]
pub trait GetUserSqlxUsecaseInterface: Send + Sync {
    async fn get_user_by_id(&self, user_id: &str) -> ApplicationResult<Option<UserResponseSqlx>>;
}

pub struct CreateUserSqlxUsecase<R: Send + Sync> {
    pub repository: R,
    pub id_generator: Box<dyn Fn() -> String + Send + Sync>,
    pub password_hasher: Box<dyn Fn(&str) -> PasswordHasherResult<String> + Send + Sync>,
}

#[async_trait]
impl<R> CreateUserSqlxUsecaseInterface for CreateUserSqlxUsecase<R>
where
    R: Send + Sync + CreateUserSqlxRepositoryInterface,
{
    async fn create_user(&self, req: UserCreateRequestSqlx) -> ApplicationResult<UserResponseSqlx> {
        let now = Utc::now().naive_utc();
        let id = (self.id_generator)();
        let password_hash = (self.password_hasher)(&req.password).map_err(|e| {
            crate::shared::error::application_error::ApplicationError::Infrastructure(e)
        })?;
        let user = UserSqlx {
            id,
            email: req.email,
            name: req.name,
            password_hash,
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
impl<R> GetUserSqlxUsecaseInterface for CreateUserSqlxUsecase<R>
where
    R: Send + Sync + CreateUserSqlxRepositoryInterface,
{
    async fn get_user_by_id(&self, user_id: &str) -> ApplicationResult<Option<UserResponseSqlx>> {
        let user_opt = self.repository.get_user_by_id(user_id).await?;
        Ok(user_opt.map(UserResponseSqlx::from))
    }
}

#[async_trait]
pub trait CreateUserSqlxRepositoryInterface: Send + Sync {
    async fn save_user(&self, user: &UserSqlx) -> ApplicationResult<UserSqlx>;
    async fn get_user_by_id(&self, user_id: &str) -> ApplicationResult<Option<UserSqlx>>;
}
