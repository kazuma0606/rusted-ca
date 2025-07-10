//main.rs
// メインアプリケーション - Webサーバー起動
// 2025/7/8

use rusted_ca::infrastructure::web::run::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 クリーンアーキテクチャ + CQRS + DIコンテナのWebサーバーを起動します");
    println!("📋 実装内容:");
    println!("  - ドメイン層: エンティティ、バリューオブジェクト、リポジトリインターフェース");
    println!("  - アプリケーション層: ユースケース、DTO、コマンド/クエリ分離");
    println!("  - インフラストラクチャ層: SQLite実装、DIコンテナ、CQRS、Webサーバー");
    println!("  - プレゼンテーション層: コントローラー、ルーター、DTO");

    // Webサーバーを起動
    run().await?;

    Ok(())
}
