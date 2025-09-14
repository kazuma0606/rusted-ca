// src/presentation/ml_controller/inference_controller.rs

use std::sync::Arc;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

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
    // --- Placeholder Implementation ---
    // This is where the presentation layer logic will go:
    // 1. Map the presentation layer DTO (`MLInferenceRequest`) to the application layer DTO (`AppInferenceRequest`).
    // 2. Call the `inference_usecase.execute()` method.
    // 3. Handle any potential errors, mapping them to appropriate HTTP status codes.
    // 4. Map the successful application layer response DTO back to a presentation layer DTO (`MLInferenceResponse`).
    // 5. Return the response as JSON with a 200 OK status code.

    println!(
        "Placeholder: Received inference request for model ID: {}",
        request.model_id
    );

    // For now, create a dummy application request and call the use case.
    let app_request = AppInferenceRequest {
        model_id: request.model_id,
        input_data: request.input_data,
    };

    match di_container.inference_usecase.execute(app_request).await {
        Ok(app_response) => {
            let presentation_response = MLInferenceResponse {
                output_data: app_response.output_data,
            };
            (StatusCode::OK, Json(presentation_response)).into_response()
        }
        Err(e) => {
            // In a real implementation, we would match on the error type
            // to return more specific status codes (e.g., 404 for model not found).
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
