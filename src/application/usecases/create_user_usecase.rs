//application/usecases/create_user_usecase.rs
// ユーザー作成ユースケース
// 2025/7/8

use crate::application::dto::user_request_dto::CreateUserRequestDto;
use crate::application::dto::user_response_dto::UserResponseDto;
use crate::domain::entity::user::User;
use crate::domain::repository::user_command_repository::UserCommandRepositoryInterface;
use crate::domain::value_object::{
    birth_date::BirthDate, email::Email, password::Password, phone::Phone, user_id::UserId,
    user_name::UserName,
};
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait CreateUserUsecaseInterface: Send + Sync {
    async fn execute(
        &self,
        request_dto: CreateUserRequestDto,
    ) -> ApplicationResult<UserResponseDto>;
}

pub struct CreateUserUseCase<T, U>
where
    T: UserCommandRepositoryInterface + Send + Sync,
    U: Fn() -> UserId + Send + Sync,
{
    command_repository: Arc<T>,
    id_generator: Arc<U>,
}

impl<T, U> CreateUserUseCase<T, U>
where
    T: UserCommandRepositoryInterface + Send + Sync,
    U: Fn() -> UserId + Send + Sync,
{
    pub fn new(command_repository: Arc<T>, id_generator: Arc<U>) -> Self {
        Self {
            command_repository,
            id_generator,
        }
    }
}

#[async_trait]
impl<T, U> CreateUserUsecaseInterface for CreateUserUseCase<T, U>
where
    T: UserCommandRepositoryInterface + Send + Sync,
    U: Fn() -> UserId + Send + Sync,
{
    async fn execute(
        &self,
        request_dto: CreateUserRequestDto,
    ) -> ApplicationResult<UserResponseDto> {
        // 1. Email重複チェック
        let email = Email::new(request_dto.email.clone()).map_err(ApplicationError::Domain)?;
        if self
            .command_repository
            .exists_by_email(&email)
            .await
            .map_err(|e| ApplicationError::Infrastructure(crate::shared::error::infrastructure_error::InfrastructureError::ResourceUnavailable {
                resource: "user".to_string(),
                message: format!("{}", e),
            }))?
        {
            return Err(ApplicationError::EmailAlreadyExists {
                email: request_dto.email,
            });
        }

        // 2. バリューオブジェクト変換
        let user_id = (self.id_generator)();
        let email = Email::new(request_dto.email.clone()).map_err(ApplicationError::Domain)?;
        let name = UserName::new(request_dto.name.clone()).map_err(ApplicationError::Domain)?;
        let password =
            Password::new(request_dto.password.clone()).map_err(ApplicationError::Domain)?;
        let phone = match &request_dto.phone {
            Some(p) => Some(Phone::new(p.clone()).map_err(ApplicationError::Domain)?),
            None => None,
        };
        let birth_date = match &request_dto.birth_date {
            Some(b) => Some(BirthDate::new(b.clone()).map_err(ApplicationError::Domain)?),
            None => None,
        };

        // 3. ドメインエンティティ生成
        let user = User::new(user_id, email, name, password, phone, birth_date)
            .map_err(ApplicationError::Domain)?;

        // 4. 保存
        self.command_repository
            .save(&user)
            .await
            .map_err(|e| ApplicationError::Infrastructure(crate::shared::error::infrastructure_error::InfrastructureError::ResourceUnavailable {
                resource: "user".to_string(),
                message: format!("{}", e),
            }))?;

        // 5. レスポンスDTO生成
        let response = UserResponseDto {
            id: user.id.0,
            email: user.email.0,
            name: user.name.0,
            phone: user.phone.map(|p| p.0),
            birth_date: user.birth_date.map(|b| b.0),
        };
        Ok(response)
    }
}
