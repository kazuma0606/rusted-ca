# Rusted-CA

## 概要
クリーンアーキテクチャ + CQRS + DI + gRPC + Discord通知対応 Rust Webアプリケーションテンプレート

## 主な機能
- ドメイン層/アプリケーション層/インフラ層/プレゼンテーション層の分離
- CQRS（コマンド・クエリ分離）
- SQLite対応
- gRPC (Prost)
- DIコンテナ
- ログ・メトリクス収集
- **Discord通知（エラー時/拡張可）**

## Discord通知機能

### 概要
- アプリケーションのHTTPエラー発生時などに、Discordの指定チャンネルへ自動通知します。
- Webhook URLを設定するだけで、チーム全体でリアルタイムに障害や重要イベントを共有できます。

### 設定方法
1. DiscordサーバーでWebhookを作成し、URLを取得
2. プロジェクトルート（Cargo.tomlと同じディレクトリ）に`.env`ファイルを作成し、以下を記載

```
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/...
DISCORD_SERVER_NAME=rusted-ca-dev
DISCORD_ENABLED=true
DISCORD_TIMEOUT=5
```

3. 必要に応じて`DISCORD_ENABLED=false`で通知を一時停止できます

### 使い方
- デフォルトでは**HTTPエラー発生時のみ**Discord通知が送信されます
- アプリケーション起動時の通知は`main.rs`の該当箇所のコメントを外すことで有効化できます
- 通知内容やタイミングは`shared/middleware/discord_middleware.rs`でカスタマイズ可能

### 拡張例
- CI/CDやテスト結果の通知も簡単に追加可能（`idea/future_task.MD`参照）

---

## 開発・運用
- `.env`は必ず「KEY=VALUE」形式で記載してください
- 詳細な設計や拡張アイデアは`ARCHITECTURE.MD`や`idea/`ディレクトリを参照

---

何か困ったことや追加要望があれば、issueやDiscordでご相談ください！
