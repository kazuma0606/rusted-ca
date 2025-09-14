// src/presentation/ml_controller/inference_controller.rs

use std::sync::Arc;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json;

use crate::application::ml_dto::inference_request::InferenceRequest as AppInferenceRequest;
use crate::application::ml_usecases::inference_usecase::InferenceUsecase;
use crate::presentation::dto::ml_inference_request::MLInferenceRequest;
use crate::presentation::dto::ml_inference_response::MLInferenceResponse;
use crate::infrastructure::di::container::DIContainer;

/// API handler for running ML inference.
pub async fn run_inference(
    State(di_container): State<Arc<DIContainer>>,
    Json(request): Json<MLInferenceRequest>,
) -> impl IntoResponse {
    // Input validation
    if request.input_data.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": {
                    "code": "INVALID_INPUT",
                    "message": "Input data cannot be empty",
                    "details": {
                        "layer": "presentation",
                        "operation": "ml_inference",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }
                }
            }))
        ).into_response();
    }

    // 1. Map presentation layer DTO to application layer DTO
    let app_request = AppInferenceRequest {
        model_id: request.model_id,
        input_data: request.input_data,
    };

    // 2. Call the inference use case
    match di_container.inference_usecase.execute(app_request).await {
        Ok(app_response) => {
            // 3. Map successful application response to presentation DTO
            let presentation_response = MLInferenceResponse {
                output_data: app_response.output_data,
            };

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "data": presentation_response,
                    "metadata": {
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "model_id": request.model_id,
                        "output_length": presentation_response.output_data.len()
                    }
                }))
            ).into_response()
        }
        Err(e) => {
            // 4. Handle errors with appropriate HTTP status codes
            use crate::shared::error::application_error::ApplicationError;

            let (status_code, error_code, message) = match &e {
                ApplicationError::NotFound { resource, id } => (
                    StatusCode::NOT_FOUND,
                    "MODEL_NOT_FOUND",
                    format!("{} with ID '{}' not found", resource, id)
                ),
                ApplicationError::ValidationFailed { field, message } => (
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_FAILED",
                    format!("Validation failed for field '{}': {}", field, message)
                ),
                ApplicationError::InternalError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    msg.clone()
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "UNKNOWN_ERROR",
                    e.to_string()
                ),
            };

            (
                status_code,
                Json(serde_json::json!({
                    "success": false,
                    "error": {
                        "code": error_code,
                        "message": message,
                        "details": {
                            "layer": "application",
                            "operation": "ml_inference",
                            "model_id": request.model_id,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }
                    }
                }))
            ).into_response()
        }
    }
}
