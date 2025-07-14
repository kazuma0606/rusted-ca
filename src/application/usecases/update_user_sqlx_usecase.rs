use crate::presentation::dto::update_user_request::UpdateUserRequest;
use crate::presentation::dto::user_response_sqlx::UserResponseSqlx;
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use chrono::Utc;

#[async_trait]
pub trait UpdateUserSqlxUsecaseInterface: Send + Sync {
    async fn update_user(
        &self,
        user_id: &str,
        req: UpdateUserRequest,
    ) -> ApplicationResult<UserResponseSqlx>;
}

pub struct UpdateUserSqlxUsecase<R: Send + Sync> {
    pub repository: R,
}

#[async_trait]
impl<R> UpdateUserSqlxUsecaseInterface for UpdateUserSqlxUsecase<R>
where
    R: Send
        + Sync
        + crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxRepositoryInterface,
{
    async fn update_user(
        &self,
        user_id: &str,
        req: UpdateUserRequest,
    ) -> ApplicationResult<UserResponseSqlx> {
        // 1. 既存ユーザー取得
        let mut user = self
            .repository
            .get_user_by_id(user_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                resource: "User".to_string(),
                id: user_id.to_string(),
            })?;

        // 2. リクエストDTOで上書き
        if let Some(name) = req.name {
            user.name = name;
        }
        if let Some(phone) = req.phone {
            user.phone = Some(phone);
        }
        if let Some(birth_date) = req.birth_date {
            user.birth_date = chrono::NaiveDate::parse_from_str(&birth_date, "%Y-%m-%d").ok();
        }
        user.updated_at = Utc::now().naive_utc();

        // 3. 更新
        let updated = self.repository.update_user(&user).await?;
        Ok(UserResponseSqlx::from(updated))
    }
}
