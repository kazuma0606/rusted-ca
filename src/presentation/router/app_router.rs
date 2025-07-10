//presentation/router/app_router.rs
// メインルーター設定
// 2025/7/8

use crate::application::usecases::create_user_usecase::CreateUserUsecaseInterface;
use crate::application::usecases::get_user_usecase::GetUserQueryUsecaseInterface;
use crate::application::usecases::update_user_usecase::UpdateUserUsecaseInterface;
use crate::presentation::controller::user_controller::UserController;
use crate::presentation::router::user_router::create_user_routes;
use axum::{Json, Router, routing::get};
use std::sync::Arc;

/// メインアプリケーションルーター
///
/// 責務:
/// 1. 全ルーターの統合
/// 2. ヘルスチェックエンドポイント
/// 3. APIプレフィックスの設定
pub fn create_app_router<T, U, V>(user_controller: Arc<UserController<T, U, V>>) -> Router
where
    T: CreateUserUsecaseInterface + Send + Sync + 'static,
    U: GetUserQueryUsecaseInterface + Send + Sync + 'static,
    V: UpdateUserUsecaseInterface + Send + Sync + 'static,
{
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/api/health",
            get(|| async {
                Json(serde_json::json!({
                    "status": "ok",
                    "message": "API is running",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }),
        )
        .nest("/api", create_user_routes(user_controller))
}
