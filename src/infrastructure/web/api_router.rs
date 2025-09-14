use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::infrastructure::di::container::DIContainer;
use crate::presentation::controller::health_controller;
use crate::presentation::router::{
    auth_router, fortune_router, grpc_router, metrics_router, user_router,
};
use crate::presentation::router::ml_router::ml_router;

pub fn build_api_router(_di_container: Arc<DIContainer>) -> Router {
    Router::new()
        .route("/health", get(health_controller::health_check))
        .nest("/auth", auth_router::create_auth_routes())
        .nest("/user", user_router::create_simple_user_routes(_di_container.clone()))
        .nest("/fortune", fortune_router::create_fortune_routes())
        .nest("/metrics", metrics_router::create_metrics_routes())
        .nest("/grpc", grpc_router::create_grpc_routes())
        .nest("/api/ml", ml_router())
}