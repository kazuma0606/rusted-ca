//application/usecases/create_account_usecase.rs
// アカウント作成ユースケース
// 2025/8/28

use crate::application::dto::account_request_dto::CreateAccountRequestDto;
use crate::application::dto::account_response_dto::AccountResponseDto;
use crate::domain::entity::account::Account;
use crate::domain::repository::account_command_repository::AccountCommandRepositoryInterface;
use crate::domain::repository::account_query_repository::AccountQueryRepositoryInterface;
use crate::domain::value_object::{Email, MerchantName};
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait CreateAccountUsecaseInterface: Send + Sync {
    async fn execute(
        &self,
        request_dto: CreateAccountRequestDto,
    ) -> ApplicationResult<AccountResponseDto>;
}

pub struct CreateAccountUsecase {
    command_repository: Arc<dyn AccountCommandRepositoryInterface>,
    query_repository: Arc<dyn AccountQueryRepositoryInterface>,
}

impl CreateAccountUsecase {
    pub fn new(
        command_repository: Arc<dyn AccountCommandRepositoryInterface>,
        query_repository: Arc<dyn AccountQueryRepositoryInterface>,
    ) -> Self {
        Self {
            command_repository,
            query_repository,
        }
    }
}

#[async_trait]
impl CreateAccountUsecaseInterface for CreateAccountUsecase {
    async fn execute(
        &self,
        request_dto: CreateAccountRequestDto,
    ) -> ApplicationResult<AccountResponseDto> {
        // 1. Business validation: Check if email already exists
        let email = Email::new(request_dto.email.clone()).map_err(|e| {
            ApplicationError::InvalidInput {
                input: request_dto.email.clone(),
                reason: e.to_string(),
            }
        })?;

        if self.query_repository.exists_by_email(&email).await.map_err(|e| {
            ApplicationError::Infrastructure(
                crate::shared::error::infrastructure_error::InfrastructureError::DatabaseOperation(
                    e.to_string(),
                ),
            )
        })? {
            return Err(ApplicationError::EmailAlreadyExists {
                email: request_dto.email,
            });
        }

        // 2. Create domain value objects
        let merchant_name = MerchantName::new(request_dto.merchant_name.clone()).map_err(|e| {
            ApplicationError::InvalidInput {
                input: request_dto.merchant_name.clone(),
                reason: e.to_string(),
            }
        })?;

        // 3. Create Account domain entity
        let account = Account::create_new(merchant_name, email, request_dto.currency_code.clone()).map_err(|e| {
            ApplicationError::InvalidInput {
                input: request_dto.currency_code,
                reason: e.to_string(),
            }
        })?;

        // 4. Persist the account
        self.command_repository.save(&account).await.map_err(|e| {
            ApplicationError::Infrastructure(
                crate::shared::error::infrastructure_error::InfrastructureError::DatabaseOperation(
                    e.to_string(),
                ),
            )
        })?;

        // 5. Return response DTO
        Ok(AccountResponseDto::from_domain(&account))
    }
}