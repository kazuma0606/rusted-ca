//application/usecases/get_account_usecase.rs
// アカウント取得ユースケース
// 2025/8/28

use crate::application::dto::account_response_dto::AccountResponseDto;
use crate::domain::repository::account_query_repository::AccountQueryRepositoryInterface;
use crate::domain::value_object::AccountId;
use crate::shared::error::application_error::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait GetAccountUsecaseInterface: Send + Sync {
    async fn execute(
        &self,
        account_id: String,
    ) -> ApplicationResult<Option<AccountResponseDto>>;
}

pub struct GetAccountUsecase {
    query_repository: Arc<dyn AccountQueryRepositoryInterface>,
}

impl GetAccountUsecase {
    pub fn new(
        query_repository: Arc<dyn AccountQueryRepositoryInterface>,
    ) -> Self {
        Self {
            query_repository,
        }
    }
}

#[async_trait]
impl GetAccountUsecaseInterface for GetAccountUsecase {
    async fn execute(
        &self,
        account_id: String,
    ) -> ApplicationResult<Option<AccountResponseDto>> {
        // 1. Validate account ID format
        let account_id_value = account_id.clone();
        let account_id = AccountId::new(account_id).map_err(|e| {
            ApplicationError::InvalidInput {
                input: account_id_value,
                reason: e.to_string(),
            }
        })?;

        // 2. Query the repository
        let account_option = self.query_repository.find_by_id(&account_id).await.map_err(|e| {
            ApplicationError::Infrastructure(
                crate::shared::error::infrastructure_error::InfrastructureError::DatabaseOperation(
                    e.to_string(),
                ),
            )
        })?;

        // 3. Convert to DTO if found
        if let Some(account) = account_option {
            Ok(Some(AccountResponseDto::from_domain(&account)))
        } else {
            Ok(None)
        }
    }
}