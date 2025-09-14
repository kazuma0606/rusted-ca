//presentation/router/metrics_router.rs
// メトリクスルーティング
// 2025/7/8

use axum::{Router, routing::get};
use crate::presentation::controller::metrics_controller;

pub fn create_metrics_routes() -> Router {
    Router::new()
        .route("/", get(|| async { "Metrics placeholder" }))
}