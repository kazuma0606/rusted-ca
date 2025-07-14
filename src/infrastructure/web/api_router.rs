//infrastructure/web/api_router.rs
// APIルーター構築
// 2025/7/13

use crate::application::usecases::create_user_sqlx_usecase::{
    CreateUserSqlxUsecaseInterface, GetUserSqlxUsecaseInterface,
};
use crate::application::usecases::delete_user_sqlx_usecase::DeleteUserSqlxUsecaseInterface;
use crate::application::usecases::update_user_sqlx_usecase::UpdateUserSqlxUsecaseInterface;

use crate::infrastructure::di::container::DIContainer;
use crate::presentation::dto::update_user_request::UpdateUserRequest;
use crate::presentation::dto::user_create_request_sqlx::UserCreateRequestSqlx;
use crate::presentation::dto::user_deleted_response::UserDeletedResponse;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

pub fn build_api_router(di: Arc<DIContainer>) -> Router {
    Router::new()
        .route("/api/user", post(create_user_handler))
        .route("/api/user/:id", get(get_user_handler))
        .route("/api/user/:id", put(update_user_handler))
        .route("/api/user/:id", delete(delete_user_handler))
        .with_state(di)
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
