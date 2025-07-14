use crate::presentation::dto::user_deleted_response::UserDeletedResponse;
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use async_trait::async_trait;

#[async_trait]
pub trait DeleteUserSqlxUsecaseInterface: Send + Sync {
    async fn delete_user(&self, user_id: &str) -> ApplicationResult<UserDeletedResponse>;
}

pub struct DeleteUserSqlxUsecase<R: Send + Sync> {
    pub repository: R,
}

#[async_trait]
impl<R> DeleteUserSqlxUsecaseInterface for DeleteUserSqlxUsecase<R>
where
    R: Send
        + Sync
        + crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxRepositoryInterface,
{
    async fn delete_user(&self, user_id: &str) -> ApplicationResult<UserDeletedResponse> {
        // 1. 事前にユーザー情報取得
        let user = self.repository.get_user_by_id(user_id).await?;
        let user = user.ok_or_else(|| ApplicationError::NotFound {
            resource: "User".to_string(),
            id: user_id.to_string(),
        })?;
        let response = UserDeletedResponse::new(user.email, user.name);
        // 2. 削除
        self.repository.delete_user(user_id).await?;
        Ok(response)
    }
}
