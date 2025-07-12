//infrastructure/web/run.rs
// Webサーバー実行関数
// 2025/7/8

use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxUsecase;
use crate::application::usecases::create_user_sqlx_usecase::CreateUserSqlxUsecaseInterface;
use crate::infrastructure::di::container::DIContainer;
use crate::presentation::dto::user_create_request_sqlx::UserCreateRequestSqlx;
use axum::{
    Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post, serve,
};
use deadpool_redis::Pool as RedisPool;
use sqlx::MySqlPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// PoC用Webサーバーを起動する
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // プール初期化
    let tidb_pool = MySqlPool::connect("mysql://root:root@localhost:3306/rusted_ca").await?;
    let redis_cfg = deadpool_redis::Config::from_url("redis://127.0.0.1/");
    let redis_pool = redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    // DI
    let di = Arc::new(DIContainer::new(tidb_pool, redis_pool));

    // ルーター
    let app = Router::new()
        .route("/api/user", post(create_user_handler))
        .with_state(di.clone());

    // サーバー起動
    println!("Listening on http://0.0.0.0:3000");
    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    Ok(())
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
