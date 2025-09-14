use reqwest::Client;
use serde_json::json;
use tokio::time::timeout;

use crate::infrastructure::config::app_config::DiscordConfig;

/// Discord通知の色定義
pub const DISCORD_COLORS: DiscordColors = DiscordColors {
    success: 0x00ff00, // 緑
    warning: 0xffff00, // 黄
    error: 0xff0000,   // 赤
    info: 0x0099ff,    // 青
};

#[derive(Debug)]
pub struct DiscordColors {
    pub success: u32,
    pub warning: u32,
    pub error: u32,
    pub info: u32,
}

/// Discord通知を送信
pub async fn send_discord_notification(
    webhook_url: &str,
    title: &str,
    message: &str,
    color: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let embed = json!({
        "title": title,
        "description": message,
        "color": color,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "footer": {
            "text": "Rusted-CA Monitoring"
        }
    });

    let payload = json!({
        "username": "Rusted-CA Bot",
        "avatar_url": "https://rust-lang.org/static/images/rust-logo-blk.svg",
        "embeds": [embed]
    });

    println!("[Discord] webhook_url: {}", webhook_url);
    println!("[Discord] payload: {}", payload);

    let res = client.post(webhook_url).json(&payload).send().await;
    println!("[Discord] response: {:?}", res);

    res?;

    Ok(())
}

/// アプリケーション起動通知
pub async fn notify_app_startup(config: &DiscordConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("[Discord] config: {:?}", config);
    if !config.enabled || config.webhook_url.is_empty() {
        println!("[Discord] 通知無効またはWebhook URL未設定");
        return Ok(());
    }

    let message = format!(
        "🚀 **Rusted-CA Application Started**\n\n\
        **Server**: {}\n\
        **Timestamp**: {}\n\
        **Status**: ✅ Running",
        config.server_name,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    // タイムアウト付きで通知
    match timeout(
        config.timeout,
        send_discord_notification(
            &config.webhook_url,
            "🛠️ Application Startup",
            &message,
            DISCORD_COLORS.success,
        ),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            eprintln!("[Discord] Discord notification timeout");
            Ok(())
        }
    }
}

/// エラー通知
pub async fn notify_error(
    config: &DiscordConfig,
    error: &str,
    context: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.enabled || config.webhook_url.is_empty() {
        return Ok(());
    }

    let message = format!(
        "🚨 **Error Detected**\n\n\
        **Error**: {}\n\
        **Context**: {}\n\
        **Timestamp**: {}",
        error,
        context,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    timeout(
        config.timeout,
        send_discord_notification(
            &config.webhook_url,
            "🚨 Critical Error",
            &message,
            DISCORD_COLORS.error,
        ),
    )
    .await
    .map_err(|_| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Discord notification timeout",
        ))
    })?
}

// discord_notification_middlewareはshared/middleware/discord_middleware.rsに移動しました。
