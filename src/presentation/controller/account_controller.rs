//presentation/controller/account_controller.rs
// Account Controller
// 2025/8/30

use crate::application::usecases::{
    create_account_usecase::{CreateAccountUseCase, CreateAccountUsecaseInterface},
    get_account_usecase::{GetAccountUseCase, GetAccountUsecaseInterface},
};
use crate::application::dto::{
    account_request_dto::CreateAccountRequestDto,
    account_response_dto::AccountResponseDto,
};

use crate::domain::value_object::{account_id::AccountId, email::Email, merchant_name::MerchantName};

use crate::presentation::dto::{
    account_create_request::AccountCreateRequest,
    account_response::AccountResponse,
    account_update_request::AccountUpdateRequest,
    account_balance_operation_request::AccountBalanceOperationRequest,
};

use crate::shared::error::presentation_error::{PresentationError, PresentationResult};
use axum::{extract::Path, Json, extract::Query, http::StatusCode, response::IntoResponse};
use crate::presentation::dto::api_response::ApiResponse;

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

    /// Update account - Not implemented
    /// PUT /api/account/{id}
    pub async fn update_account(
        &self,
        Path(_id): Path<String>,
        Json(_request): Json<AccountUpdateRequest>,
    ) -> PresentationResult<Json<ApiResponse<AccountResponse>>> {
        Err(PresentationError::InternalServer {
            message: "Update account functionality is not yet implemented".to_string(),
        })
    }

    /// Delete account - Not implemented
    /// DELETE /api/account/{id}
    pub async fn delete_account(
        &self,
        Path(_id): Path<String>,
    ) -> PresentationResult<Json<ApiResponse<AccountResponse>>> {
        Err(PresentationError::InternalServer {
            message: "Delete account functionality is not yet implemented".to_string(),
        })
    }

    /// Get account balance - Not implemented
    /// GET /api/account/{id}/balance
    pub async fn get_account_balance(
        &self,
        Path(_id): Path<String>,
    ) -> PresentationResult<Json<ApiResponse<AccountResponse>>> {
        Err(PresentationError::InternalServer {
            message: "Get account balance functionality is not yet implemented".to_string(),
        })
    }

    /// Credit account - Not implemented
    /// POST /api/account/{id}/credit
    pub async fn credit_account(
        &self,
        Path(_id): Path<String>,
        Json(_request): Json<AccountBalanceOperationRequest>,
    ) -> PresentationResult<Json<ApiResponse<AccountResponse>>> {
        Err(PresentationError::InternalServer {
            message: "Credit account functionality is not yet implemented".to_string(),
        })
    }

    /// Debit account - Not implemented
    /// POST /api/account/{id}/debit
    pub async fn debit_account(
        &self,
        Path(_id): Path<String>,
        Json(_request): Json<AccountBalanceOperationRequest>,
    ) -> PresentationResult<Json<ApiResponse<AccountResponse>>> {
        Err(PresentationError::InternalServer {
            message: "Debit account functionality is not yet implemented".to_string(),
        })
    }
}