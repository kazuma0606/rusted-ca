//infrastructure/web/run.rs
// Webサーバー実行関数
// 2025/7/8

use axum::{
    Router,
    routing::{get, post},
    serve,
};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::infrastructure::di::container::DIContainer;
use crate::presentation::router::app_router::create_app_router;

/// Webサーバーを起動する
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. DIコンテナの初期化
    let di_container = DIContainer::new();

    // 2. データベース接続のテスト
    let db_connection = di_container.create_database_connection()?;
    println!("✅ データベース接続の作成が完了しました");

    // 3. Repository実装のテスト
    let (command_repo, query_repo) = di_container.create_repositories()?;
    println!("✅ Repository実装の作成が完了しました");

    // 4. ID生成器のテスト
    let id_generator = di_container.create_id_generator();
    let test_id = id_generator();
    println!("✅ ID生成器のテストが完了しました: {}", test_id.0);

    // 5. ルーティング設定
    let user_controller = di_container.build_user_controller()?;
    let app = create_app_router(user_controller);

    // 6. サーバー起動
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("🚀 Server starting on {}", addr);
    println!("📋 利用可能なエンドポイント:");
    println!("  - GET  /health - ヘルスチェック");
    println!("  - GET  /api/health - APIヘルスチェック");
    println!("  - POST /api/users - ユーザー作成");
    println!("  - GET  /api/users/:id - ユーザー取得");
    println!("  - PUT  /api/users/:id - ユーザー更新");

    serve(listener, app).await?;

    Ok(())
}

/// APIヘルスチェックエンドポイント
async fn api_health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "message": "API is running",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
