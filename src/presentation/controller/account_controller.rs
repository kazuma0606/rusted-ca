//presentation/controller/account_controller.rs
// Account HTTP Controller
// 2025/8/28

use crate::application::usecases::{
    create_account_usecase::{CreateAccountUseCase, CreateAccountUsecaseInterface},
    get_account_usecase::{GetAccountUseCase, GetAccountUsecaseInterface},
};
use crate::application::dto::{
    account_request_dto::CreateAccountRequestDto,
    account_response_dto::AccountResponseDto,
};
use crate::domain::value_object::{AccountId, AccountStatus, Email, MerchantName, Money};
use crate::presentation::dto::{
    account_balance_operation_request::AccountBalanceOperationRequest,
    account_create_request::AccountCreateRequest,
    account_response::{AccountResponse, BalanceOnlyResponse},
    account_update_request::AccountUpdateRequest,
    api_response::ApiResponse,
};
use crate::shared::error::presentation_error::{PresentationError, PresentationResult};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AccountController {
    create_account_usecase: Arc<dyn CreateAccountUsecaseInterface>,
    get_account_usecase: Arc<dyn GetAccountUsecaseInterface>,
}

impl AccountController {
    pub fn new(
        create_account_usecase: Arc<dyn CreateAccountUsecaseInterface>,
        get_account_usecase: Arc<dyn GetAccountUsecaseInterface>,
    ) -> Self {
        Self {
            create_account_usecase,
            get_account_usecase,
        }
    }

    /// Create a new account
    /// POST /api/account
    pub async fn create_account(
        &self,
        Json(request): Json<AccountCreateRequest>,
    ) -> PresentationResult<Json<ApiResponse<AccountResponse>>> {
        // Convert Presentation DTO to Application DTO
        let app_dto = CreateAccountRequestDto {
            merchant_name: request.merchant_name,
            email: request.email,
            currency_code: request.currency_code,
        };

        // Execute use case
        let result = self
            .create_account_usecase
            .execute(app_dto)
            .await
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?;

        // Convert Application DTO to Presentation DTO
        let response = AccountResponse::from_app_dto(result);
        Ok(Json(ApiResponse::success(response)))
    }

    /// Get account by ID
    /// GET /api/account/{id}
    pub async fn get_account(
        &self,
        Path(id): Path<String>,
    ) -> PresentationResult<Json<ApiResponse<AccountResponse>>> {
        let account_dto = self
            .get_account_usecase
            .execute(id.clone())
            .await
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?;

        match account_dto {
            Some(dto) => {
                let response = AccountResponse::from_app_dto(dto);
                Ok(Json(ApiResponse::success(response)))
            }
            None => Err(PresentationError::NotFound {
                resource: format!("Account with id {}", id),
            }),
        }
    }

    /// Update account
    /// PUT /api/account/{id}
    pub async fn update_account(
        &self,
        Path(id): Path<String>,
        Json(request): Json<AccountUpdateRequest>,
    ) -> PresentationResult<Json<ApiResponse<AccountResponse>>> {
        if request.is_empty() {
            return Err(PresentationError::BadRequest {
                message: "At least one field must be provided for update".to_string(),
            });
        }

        let account_id = AccountId::new(id)
            .map_err(|e| PresentationError::BadRequest {
                message: format!("Invalid id: {}", e.to_string()),
            })?;

        // Get current account
        let mut account = self
            .get_account_usecase
            .execute(account_id)
            .await
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?
            .ok_or_else(|| PresentationError::NotFound {
                resource: format!("Account with id {}", id),
            })?;

        // Apply updates
        if let Some(merchant_name_str) = request.merchant_name {
            let merchant_name = MerchantName::new(merchant_name_str)
                .map_err(|e| PresentationError::BadRequest {
                    message: format!("Invalid merchant_name: {}", e),
                })?;
            
            account.update_merchant_info(merchant_name)
                .map_err(|e| PresentationError::InternalServer {
                    message: e.to_string(),
                })?;
        }

        if let Some(status_str) = request.status {
            let status = AccountStatus::from_string(&status_str)
                .map_err(|e| PresentationError::BadRequest {
                    message: format!("Invalid status: {}", e),
                })?;
            
            account.update_status(status)
                .map_err(|e| PresentationError::InternalServer {
                    message: e.to_string(),
                })?;
        }

        // TODO: Save updated account through use case
        // For now, return the updated account
        let response = AccountResponse::from_account(&account);
        Ok(Json(ApiResponse::success(response)))
    }

    /// Delete account
    /// DELETE /api/account/{id}
    pub async fn delete_account(
        &self,
        Path(id): Path<String>,
    ) -> PresentationResult<Json<ApiResponse<()>>> {
        let account_id = AccountId::new(id)
            .map_err(|e| PresentationError::BadRequest {
                message: format!("Invalid id: {}", e.to_string()),
            })?;

        // Get current account to check if it can be closed
        let mut account = self
            .get_account_usecase
            .execute(account_id)
            .await
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?
            .ok_or_else(|| PresentationError::NotFound {
                resource: format!("Account with id {}", id),
            })?;

        // Close the account (business rule: must have zero balance)
        account.close()
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?;

        // TODO: Save closed account through use case
        Ok(Json(ApiResponse::success(())))
    }

    /// Get account balance
    /// GET /api/account/{id}/balance
    pub async fn get_account_balance(
        &self,
        Path(id): Path<String>,
    ) -> PresentationResult<Json<ApiResponse<BalanceOnlyResponse>>> {
        let account_id = AccountId::new(id)
            .map_err(|e| PresentationError::BadRequest {
                message: format!("Invalid id: {}", e.to_string()),
            })?;

        let account = self
            .get_account_usecase
            .execute(account_id)
            .await
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?
            .ok_or_else(|| PresentationError::NotFound {
                resource: format!("Account with id {}", id),
            })?;

        let response = BalanceOnlyResponse::from_account(&account);
        Ok(Json(ApiResponse::success(response)))
    }

    /// Credit money to account
    /// POST /api/account/{id}/credit
    pub async fn credit_account(
        &self,
        Path(id): Path<String>,
        Json(request): Json<AccountBalanceOperationRequest>,
    ) -> PresentationResult<Json<ApiResponse<BalanceOnlyResponse>>> {
        let account_id = AccountId::new(id)
            .map_err(|e| PresentationError::BadRequest {
                message: format!("Invalid id: {}", e.to_string()),
            })?;

        // Get current account
        let mut account = self
            .get_account_usecase
            .execute(account_id)
            .await
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?
            .ok_or_else(|| PresentationError::NotFound {
                resource: format!("Account with id {}", id),
            })?;

        // Create money amount
        let credit_amount = Money::from_major_units(request.amount, request.currency_code)
            .map_err(|e| PresentationError::BadRequest {
                message: format!("Invalid amount: {}", e),
            })?;

        // Credit the account
        account.credit(&credit_amount)
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?;

        // TODO: Save updated account through use case
        let response = BalanceOnlyResponse::from_account(&account);
        Ok(Json(ApiResponse::success(response)))
    }

    /// Debit money from account
    /// POST /api/account/{id}/debit
    pub async fn debit_account(
        &self,
        Path(id): Path<String>,
        Json(request): Json<AccountBalanceOperationRequest>,
    ) -> PresentationResult<Json<ApiResponse<BalanceOnlyResponse>>> {
        let account_id = AccountId::new(id)
            .map_err(|e| PresentationError::BadRequest {
                message: format!("Invalid id: {}", e.to_string()),
            })?;

        // Get current account
        let mut account = self
            .get_account_usecase
            .execute(account_id)
            .await
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?
            .ok_or_else(|| PresentationError::NotFound {
                resource: format!("Account with id {}", id),
            })?;

        // Create money amount
        let debit_amount = Money::from_major_units(request.amount, request.currency_code)
            .map_err(|e| PresentationError::BadRequest {
                message: format!("Invalid amount: {}", e),
            })?;

        // Debit from the account
        account.debit(&debit_amount)
            .map_err(|e| PresentationError::InternalServer {
                message: e.to_string(),
            })?;

        // TODO: Save updated account through use case
        let response = BalanceOnlyResponse::from_account(&account);
        Ok(Json(ApiResponse::success(response)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Note: These would be integration tests that require proper DI setup
    // The actual test implementation would require mocking the use cases
}