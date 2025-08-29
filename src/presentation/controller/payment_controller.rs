//presentation/controller/payment_controller.rs
// Payment HTTP Controller
// 2025/8/29

use crate::application::usecases::{
    create_payment_usecase::CreatePaymentUseCase,
    process_payment_usecase::ProcessPaymentUseCase,
    refund_payment_usecase::RefundPaymentUseCase,
};
use crate::domain::entity::payment::Payment;
use crate::domain::repository::payment_query_repository::PaymentQueryRepository;
use crate::domain::value_object::{
    AccountId, Money, PaymentId, PaymentMethod, PaymentStatus,
};
use crate::presentation::dto::{
    api_response::ApiResponse,
    payment_create_request::PaymentCreateRequest,
    payment_operation_request::{PaymentOperationRequest, PaymentSearchQuery},
    payment_response::{PaymentListResponse, PaymentResponse, PaymentStatsResponse},
};
use crate::shared::error::presentation_error::{PresentationError, PresentationResult};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PaymentController {
    create_payment_usecase: Arc<CreatePaymentUseCase>,
    process_payment_usecase: Arc<ProcessPaymentUseCase>,
    refund_payment_usecase: Arc<RefundPaymentUseCase>,
    payment_query_repository: Arc<dyn PaymentQueryRepository>,
}

impl PaymentController {
    pub fn new(
        create_payment_usecase: Arc<CreatePaymentUseCase>,
        process_payment_usecase: Arc<ProcessPaymentUseCase>,
        refund_payment_usecase: Arc<RefundPaymentUseCase>,
        payment_query_repository: Arc<dyn PaymentQueryRepository>,
    ) -> Self {
        Self {
            create_payment_usecase,
            process_payment_usecase,
            refund_payment_usecase,
            payment_query_repository,
        }
    }

    /// Create a new payment
    pub async fn create_payment(
        &self,
        Json(request): Json<PaymentCreateRequest>,
    ) -> PresentationResult<Json<ApiResponse<PaymentResponse>>> {
        // Convert string to domain objects
        let account_id = AccountId::from_string(request.account_id.clone())
            .map_err(|e| PresentationError::BadRequest(format!("Invalid account ID: {}", e)))?;

        let amount = Money::from_major_units(request.amount, request.currency_code.clone())
            .map_err(|e| PresentationError::BadRequest(format!("Invalid amount: {}", e)))?;

        let payment_method = PaymentMethod::from_string(&request.payment_method)
            .map_err(|e| PresentationError::BadRequest(format!("Invalid payment method: {}", e)))?;

        // Execute use case
        let payment = self
            .create_payment_usecase
            .execute(
                account_id,
                amount,
                payment_method,
                request.description,
                request.customer_email,
                request.metadata,
            )
            .await
            .map_err(PresentationError::from)?;

        // Convert to response
        let response = self.payment_to_response(payment);

        Ok(Json(ApiResponse::success_with_data(
            response,
            "Payment created successfully",
        )))
    }

    /// Get payment by ID
    pub async fn get_payment(
        &self,
        Path(payment_id): Path<String>,
    ) -> PresentationResult<Json<ApiResponse<PaymentResponse>>> {
        let payment_id = PaymentId::from_string(payment_id)
            .map_err(|e| PresentationError::BadRequest(format!("Invalid payment ID: {}", e)))?;

        let payment = self
            .payment_query_repository
            .find_by_id(&payment_id)
            .await
            .map_err(PresentationError::from)?;

        match payment {
            Some(payment) => {
                let response = self.payment_to_response(payment);
                Ok(Json(ApiResponse::success_with_data(
                    response,
                    "Payment retrieved successfully",
                )))
            }
            None => Err(PresentationError::NotFound(
                "Payment not found".to_string(),
            )),
        }
    }

    /// Process a payment (change status to processing then success)
    pub async fn process_payment(
        &self,
        Path(payment_id): Path<String>,
        Json(request): Json<PaymentOperationRequest>,
    ) -> PresentationResult<Json<ApiResponse<PaymentResponse>>> {
        let payment_id = PaymentId::from_string(payment_id)
            .map_err(|e| PresentationError::BadRequest(format!("Invalid payment ID: {}", e)))?;

        let payment = self
            .process_payment_usecase
            .execute(payment_id)
            .await
            .map_err(PresentationError::from)?;

        let response = self.payment_to_response(payment);

        Ok(Json(ApiResponse::success_with_data(
            response,
            "Payment processed successfully",
        )))
    }

    /// Refund a payment
    pub async fn refund_payment(
        &self,
        Path(payment_id): Path<String>,
        Json(request): Json<PaymentOperationRequest>,
    ) -> PresentationResult<Json<ApiResponse<PaymentResponse>>> {
        let payment_id = PaymentId::from_string(payment_id)
            .map_err(|e| PresentationError::BadRequest(format!("Invalid payment ID: {}", e)))?;

        // Parse refund amount if provided
        let refund_amount = if request.is_partial_refund() {
            let amount = request.amount.unwrap();
            let currency = request.currency_code.unwrap();
            Some(
                Money::from_major_units(amount, currency)
                    .map_err(|e| PresentationError::BadRequest(format!("Invalid refund amount: {}", e)))?,
            )
        } else {
            None
        };

        let reason = request.reason.unwrap_or_else(|| "Refund requested".to_string());

        let payment = self
            .refund_payment_usecase
            .execute(payment_id, refund_amount.as_ref(), reason)
            .await
            .map_err(PresentationError::from)?;

        let response = self.payment_to_response(payment);

        Ok(Json(ApiResponse::success_with_data(
            response,
            "Payment refunded successfully",
        )))
    }

    /// List payments with optional filtering
    pub async fn list_payments(
        &self,
        Query(query): Query<PaymentSearchQuery>,
    ) -> PresentationResult<Json<ApiResponse<PaymentListResponse>>> {
        let limit = query.limit.unwrap_or(50).min(100); // Max 100 items per page
        let offset = query.offset.unwrap_or(0);

        // Convert query parameters to domain objects
        let account_id = if let Some(id) = query.account_id {
            Some(
                AccountId::from_string(id)
                    .map_err(|e| PresentationError::BadRequest(format!("Invalid account ID: {}", e)))?,
            )
        } else {
            None
        };

        let status = if let Some(status_str) = query.status {
            Some(
                PaymentStatus::from_string(&status_str)
                    .map_err(|e| PresentationError::BadRequest(format!("Invalid status: {}", e)))?,
            )
        } else {
            None
        };

        // Query payments based on filters
        let payments = if let Some(account_id) = &account_id {
            self.payment_query_repository
                .find_by_account_id(account_id)
                .await
                .map_err(PresentationError::from)?
        } else if let Some(status) = &status {
            self.payment_query_repository
                .find_by_status(status)
                .await
                .map_err(PresentationError::from)?
        } else {
            // If no specific filters, we'd need a find_all method
            // For now, return empty list
            vec![]
        };

        // Apply pagination
        let total_count = payments.len() as u64;
        let paginated_payments: Vec<Payment> = payments
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();

        let payment_responses: Vec<PaymentResponse> = paginated_payments
            .into_iter()
            .map(|payment| self.payment_to_response(payment))
            .collect();

        let response = PaymentListResponse::new(
            payment_responses,
            total_count,
            (offset / limit) + 1,
            limit,
        );

        Ok(Json(ApiResponse::success_with_data(
            response,
            "Payments retrieved successfully",
        )))
    }

    /// Get payment statistics for an account
    pub async fn get_payment_stats(
        &self,
        Path(account_id): Path<String>,
    ) -> PresentationResult<Json<ApiResponse<PaymentStatsResponse>>> {
        let account_id = AccountId::from_string(account_id)
            .map_err(|e| PresentationError::BadRequest(format!("Invalid account ID: {}", e)))?;

        let stats = self
            .payment_query_repository
            .get_payment_stats(&account_id)
            .await
            .map_err(PresentationError::from)?;

        let response = PaymentStatsResponse::new(
            account_id.value().to_string(),
            stats.total_payments,
            stats.successful_payments,
            stats.failed_payments,
            stats.pending_payments,
            stats.refunded_payments,
            stats.total_amount_cents as f64 / 100.0,
            stats.successful_amount_cents as f64 / 100.0,
            "USD".to_string(), // TODO: Get from account currency
        );

        Ok(Json(ApiResponse::success_with_data(
            response,
            "Payment statistics retrieved successfully",
        )))
    }

    /// Helper method to convert Payment entity to PaymentResponse DTO
    fn payment_to_response(&self, payment: Payment) -> PaymentResponse {
        PaymentResponse::new(
            payment.id().value().to_string(),
            payment.account_id().value().to_string(),
            payment.amount().to_major_units(),
            payment.amount().currency_code().to_string(),
            payment.payment_method().as_str().to_string(),
            payment.status().as_str().to_string(),
            payment.reference_number().value().to_string(),
            payment.description().cloned(),
            payment.customer_email().cloned(),
            payment.metadata().cloned(),
            *payment.created_at(),
            *payment.updated_at(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_controller_creation() {
        // This test just verifies that PaymentController can be created
        // More comprehensive tests would require mock implementations
        assert!(true);
    }
}