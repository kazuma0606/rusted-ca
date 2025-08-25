//infrastructure/web/api_router.rs
// APIルーター構築
// 2025/7/13

use crate::application::usecases::create_user_sqlx_usecase::{
    CreateUserSqlxUsecaseInterface, GetUserSqlxUsecaseInterface,
};
use crate::application::usecases::delete_user_sqlx_usecase::DeleteUserSqlxUsecaseInterface;
use crate::application::usecases::update_user_sqlx_usecase::UpdateUserSqlxUsecaseInterface;

use crate::infrastructure::di::container::DIContainer;

use crate::presentation::dto::log_search_request::LogSearchRequest;
use crate::presentation::dto::update_user_request::UpdateUserRequest;
use crate::presentation::dto::user_create_request_sqlx::UserCreateRequestSqlx;
use crate::shared::middleware::logging_middleware::LoggingLayer;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use tower::ServiceBuilder;

pub fn build_api_router(di: Arc<DIContainer>) -> Router<Arc<DIContainer>> {
    Router::new()
        // 既存のユーザー管理API
        .route("/api/user", post(create_user_handler))
        .route("/api/user/:id", get(get_user_handler))
        .route("/api/user/:id", put(update_user_handler))
        .route("/api/user/:id", delete(delete_user_handler))
        // ヘルスチェックAPI
        .route("/health", get(health_check_handler))
        // ログ管理API（直接ハンドラーを追加）
        .route("/api/logs/search", get(log_search_handler))
        .route("/api/logs/logs/:id", get(log_get_by_id_handler))
        .route("/api/logs/dashboard", get(log_dashboard_handler))
        // ミドルウェアスタック（順序重要）
        .layer(
            ServiceBuilder::new().layer(LoggingLayer::new(
                di.collect_log_usecase.clone(),
                di.uuid_generator.clone(),
            )), // 将来的に他のミドルウェア（メトリクス、セキュリティヘッダーなど）を追加
        )
        .with_state(di)
}

async fn health_check_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

async fn create_user_handler(
    State(di): State<Arc<DIContainer>>,
    Json(payload): Json<UserCreateRequestSqlx>,
) -> impl IntoResponse {
    match di.create_user_usecase.create_user(payload).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_user_handler(
    State(di): State<Arc<DIContainer>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match di.create_user_usecase.get_user_by_id(&user_id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn update_user_handler(
    State(di): State<Arc<DIContainer>>,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    match di.update_user_usecase.update_user(&user_id, payload).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn delete_user_handler(
    State(di): State<Arc<DIContainer>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match di.delete_user_usecase.delete_user(&user_id).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ログ管理ハンドラー関数
async fn log_search_handler(
    State(di): State<Arc<DIContainer>>,
    Query(request): Query<LogSearchRequest>,
) -> impl IntoResponse {
    match crate::presentation::controller::log_controller::LogController::search_logs(
        State(di.log_controller.clone()),
        Query(request),
    )
    .await
    {
        Ok(response) => response.into_response(),
        Err(status) => status.into_response(),
    }
}

async fn log_get_by_id_handler(
    State(di): State<Arc<DIContainer>>,
    Path(log_id): Path<String>,
) -> impl IntoResponse {
    match crate::presentation::controller::log_controller::LogController::get_log_by_id(
        State(di.log_controller.clone()),
        Path(log_id),
    )
    .await
    {
        Ok(response) => response.into_response(),
        Err(status) => status.into_response(),
    }
}

async fn log_dashboard_handler() -> impl IntoResponse {
    crate::presentation::controller::log_controller::LogController::log_dashboard().await
}
