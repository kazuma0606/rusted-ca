//main.rs
// メインアプリケーション - Webサーバー起動
// 2025/7/8

use rusted_ca::infrastructure::config::app_config::AppConfig;
use rusted_ca::infrastructure::web::run::run;
// use rusted_ca::shared::notification::discord_notification::notify_app_startup;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dotenv_result = dotenvy::from_path("C:/Users/yoshi/rusted-ca/rusted-ca/.env");
    println!("DEBUG: dotenvy result = {:?}", dotenv_result);

    for (key, value) in std::env::vars() {
        println!("ENV: {} = {}", key, value);
    }

    println!("🚀 クリーンアーキテクチャ + CQRS + DIコンテナのWebサーバーを起動します");
    println!("📋 実装内容:");
    println!("  - ドメイン層: エンティティ、バリューオブジェクト、リポジトリインターフェース");
    println!("  - アプリケーション層: ユースケース、DTO、コマンド/クエリ分離");
    println!("  - インフラストラクチャ層: SQLite実装、DIコンテナ、CQRS、Webサーバー");
    println!("  - プレゼンテーション層: コントローラー、ルーター、DTO");
    println!("  - gRPCサーバー: Protocol Buffers + Prost + Axum");
    println!("  - Discord通知: リアルタイム監視・アラート");

    // アプリケーション設定を読み込み
    let app_config = AppConfig::from_env();
    println!("DEBUG: DISCORD_ENABLED = {}", app_config.discord.enabled);
    println!(
        "DEBUG: DISCORD_WEBHOOK_URL = {}",
        app_config.discord.webhook_url
    );
    println!(
        "DEBUG: DISCORD_SERVER_NAME = {}",
        app_config.discord.server_name
    );

    // Discord通知を送信（起動時通知は一旦コメントアウト）
    /*
    if app_config.discord.enabled {
        println!("DEBUG: notify_app_startup will be called");
        if let Err(e) = notify_app_startup(&app_config.discord).await {
            eprintln!("Failed to send Discord startup notification: {}", e);
        }
    } else {
        println!("DEBUG: notify_app_startup will NOT be called");
    }
    */

    // HTTP + gRPC統合サーバーを起動
    run().await?;

    Ok(())
}
