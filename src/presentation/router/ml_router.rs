// src/presentation/router/ml_router.rs

use axum::{routing::post, Router};
use std::sync::Arc;

use crate::infrastructure::di::container::DIContainer;
use crate::presentation::ml_controller::inference_controller;

/// Creates a router for ML-related API endpoints.
pub fn ml_router() -> Router<Arc<DIContainer>> {
    Router::new()
        .route("/predict", post(inference_controller::run_inference))
}
