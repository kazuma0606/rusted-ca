//infrastructure/web/api_router.rs
// APIルーター構築
// 2025/7/13

use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxUsecaseInterface;
use crate::infrastructure::di::container::DIContainer;
use crate::presentation::dto::user_create_request_sqlx::UserCreateRequestSqlx;
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use std::sync::Arc;

pub fn build_api_router(di: Arc<DIContainer>) -> Router {
    Router::new()
        .route("/api/user", post(create_user_handler))
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
