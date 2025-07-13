# Redis データ構造 完全活用ガイド

## 概要

Redisの主要データ構造（String/JSON、Hash、Streams、Pub/Sub）の特徴、適用ユースケース、Rustでの実装難易度を網羅的にまとめたリファレンスです。

---

## 1. String / JSON

### 特徴
- **最もシンプル**なKey-Valueストレージ
- バイナリセーフ（画像、ファイルも保存可能）
- **RedisJSON**使用時はJSON操作が可能
- **原子的操作**（INCR、APPEND等）対応

### 適用データ
- セッション情報
- キャッシュデータ
- 設定値・フラグ
- カウンター
- 一時的なトークン

### ユースケース

#### ✅ 向いている
- **Webセッション管理**
- **APIレスポンスキャッシュ**
- **認証トークン保存**
- **カウンター（PV、いいね数）**
- **機能フラグ・設定管理**

#### ❌ 向いていない
- 部分更新が頻繁なデータ
- 複雑な検索が必要なデータ
- リレーショナルなデータ

### Rustでの使いやすさ：⭐⭐⭐⭐⭐

```rust
// 非常にシンプル
async fn cache_user_session(redis: &Connection, user_id: Uuid, session: &Session) -> Result<()> {
    let key = format!("session:{}", user_id);
    let json = serde_json::to_string(session)?;
    redis.set_ex(&key, &json, 3600).await?; // 1時間TTL
    Ok(())
}

async fn get_cached_session(redis: &Connection, user_id: Uuid) -> Result<Option<Session>> {
    let key = format!("session:{}", user_id);
    let json: Option<String> = redis.get(&key).await?;
    match json {
        Some(data) => Ok(Some(serde_json::from_str(&data)?)),
        None => Ok(None),
    }
}

// RedisJSON使用（より高速）
async fn update_user_profile_json(redis: &Connection, user_id: Uuid, name: &str) -> Result<()> {
    let key = format!("user:{}", user_id);
    redis::cmd("JSON.SET")
        .arg(&key)
        .arg("$.name")
        .arg(name)
        .query_async(redis).await?;
    Ok(())
}
```

**メリット**
- Serdeで自動シリアライゼーション
- エラー処理がシンプル
- TTL設定が簡単

**デメリット**
- 大きなオブジェクトの部分更新は非効率

---

## 2. Hash

### 特徴
- **フィールド-値**のマッピング（構造体ライク）
- **メモリ効率**が最高
- フィールド単位での操作が可能
- **原子的**なフィールド操作

### 適用データ
- ユーザープロフィール
- 商品情報
- 設定オブジェクト
- メトリクス・統計データ
- キャッシュされたエンティティ

### ユースケース

#### ✅ 向いている
- **ユーザープロフィール管理**
- **商品カタログキャッシュ**
- **リアルタイムメトリクス**
- **頻繁に部分更新されるデータ**
- **構造化されたキャッシュ**

#### ❌ 向いていない
- ネストした複雑な構造
- 大量のフィールド（1000+）
- 時系列データ

### Rustでの使いやすさ：⭐⭐⭐

```rust
// 手動マッピングが必要だが高性能
async fn save_user_profile(redis: &Connection, user: &User) -> Result<()> {
    let key = format!("user:{}", user.id);
    
    redis::pipe()
        .hset(&key, "id", user.id.to_string())
        .hset(&key, "email", user.email.as_deref().unwrap_or(""))
        .hset(&key, "name", user.name.as_deref().unwrap_or(""))
        .hset(&key, "last_login", user.last_login.map(|t| t.timestamp()).unwrap_or(0))
        .hset(&key, "status", user.status.to_string())
        .expire(&key, 3600)
        .query_async(redis).await?;
    Ok(())
}

async fn get_user_profile(redis: &Connection, user_id: Uuid) -> Result<Option<User>> {
    let key = format!("user:{}", user_id);
    let hash: HashMap<String, String> = redis.hgetall(&key).await?;
    
    if hash.is_empty() {
        return Ok(None);
    }
    
    // 手動でデシリアライズ（面倒だが高速）
    let user = User {
        id: Uuid::parse_str(&hash["id"])?,
        email: hash.get("email").filter(|s| !s.is_empty()).cloned(),
        name: hash.get("name").filter(|s| !s.is_empty()).cloned(),
        last_login: hash.get("last_login")
            .and_then(|s| s.parse::<i64>().ok())
            .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
        status: hash.get("status")
            .and_then(|s| UserStatus::from_str(s).ok())
            .unwrap_or_default(),
    };
    Ok(Some(user))
}

// フィールド単位の高速更新
async fn update_last_login(redis: &Connection, user_id: Uuid) -> Result<()> {
    let key = format!("user:{}", user_id);
    let timestamp = Utc::now().timestamp();
    redis.hset(&key, "last_login", timestamp).await?;
    Ok(())
}
```

**メリット**
- 最高のメモリ効率
- フィールド単位操作が高速
- 部分更新が簡単

**デメリット**
- 手動型変換が必要
- Option型の扱いが面倒
- ネストした構造は不可

---

## 3. Streams

### 特徴
- **時系列データ**専用
- **自動ID生成**（タイムスタンプベース）
- **コンシューマーグループ**で分散処理
- **永続化**された順序保証
- **Range検索**対応

### 適用データ
- イベントログ
- ユーザーアクティビティ
- 監査ログ
- メッセージキュー
- センサーデータ

### ユースケース

#### ✅ 向いている
- **イベントソーシング**
- **ユーザー行動分析**
- **監査ログ・コンプライアンス**
- **非同期タスクキュー**
- **リアルタイム分析**
- **チャット・通知システム**

#### ❌ 向いていない
- 単純なキャッシュ
- 静的なマスターデータ
- 頻繁な削除・更新が必要なデータ

### Rustでの使いやすさ：⭐⭐

```rust
// イベントの追加は簡単
async fn log_user_activity(redis: &Connection, activity: &UserActivity) -> Result<String> {
    let stream_key = "user_activities";
    
    let entry_id: String = redis::cmd("XADD")
        .arg(stream_key)
        .arg("*") // 自動ID生成
        .arg("user_id").arg(activity.user_id.to_string())
        .arg("action").arg(&activity.action)
        .arg("timestamp").arg(activity.timestamp.to_rfc3339())
        .arg("metadata").arg(serde_json::to_string(&activity.metadata)?)
        .query_async(redis).await?;
    
    Ok(entry_id)
}

// Range検索（時系列分析）
async fn get_user_activities_in_range(
    redis: &Connection, 
    start_time: DateTime<Utc>, 
    end_time: DateTime<Utc>
) -> Result<Vec<UserActivity>> {
    let start_id = format!("{}-0", start_time.timestamp_millis());
    let end_id = format!("{}-0", end_time.timestamp_millis());
    
    let entries: Vec<redis::Value> = redis::cmd("XRANGE")
        .arg("user_activities")
        .arg(start_id)
        .arg(end_id)
        .query_async(redis).await?;
    
    // パース処理（複雑）
    let activities = parse_stream_entries(entries)?;
    Ok(activities)
}

// コンシューマーグループでの分散処理
async fn process_activities_worker(redis: &Connection, worker_id: &str) -> Result<()> {
    loop {
        let messages: Vec<redis::Value> = redis::cmd("XREADGROUP")
            .arg("GROUP").arg("activity_processors").arg(worker_id)
            .arg("COUNT").arg(10)
            .arg("STREAMS").arg("user_activities").arg(">")
            .query_async(redis).await?;
        
        for message in messages {
            // メッセージ処理
            process_activity_message(message).await?;
            
            // 処理完了をACK
            redis::cmd("XACK")
                .arg("user_activities")
                .arg("activity_processors")
                .arg(message_id)
                .query_async(redis).await?;
        }
    }
}
```

**メリット**
- 時系列データの完璧な管理
- 分散処理のネイティブサポート
- 障害時の復旧機能

**デメリット**
- パース処理が複雑
- 学習コストが高い
- デバッグが困難

---

## 4. Pub/Sub

### 特徴
- **リアルタイム**メッセージング
- **多対多**通信
- **永続化されない**（揮発性）
- **パターンマッチング**サブスクライブ
- **非同期**通信

### 適用データ
- リアルタイム通知
- ライブアップデート
- イベント配信
- キャッシュ無効化信号
- システム間連携

### ユースケース

#### ✅ 向いている
- **リアルタイム通知システム**
- **ライブチャット**
- **WebSocketブロードキャスト**
- **キャッシュ無効化**
- **マイクロサービス間通信**
- **ライブダッシュボード**

#### ❌ 向いていない
- 確実な配信が必要なメッセージ
- 永続化が必要なデータ
- 順序保証が重要なデータ

### Rustでの使いやすさ：⭐⭐⭐⭐

```rust
// パブリッシャー（送信側）
async fn notify_user_update(redis: &Connection, user_id: Uuid, event: &str) -> Result<()> {
    let channel = format!("user_updates:{}", user_id);
    let message = serde_json::to_string(&UserUpdateEvent {
        user_id,
        event: event.to_string(),
        timestamp: Utc::now(),
    })?;
    
    redis.publish(&channel, &message).await?;
    Ok(())
}

// 全体ブロードキャスト
async fn broadcast_system_maintenance(redis: &Connection) -> Result<()> {
    let message = SystemMessage {
        level: "warning".to_string(),
        content: "System maintenance in 10 minutes".to_string(),
        timestamp: Utc::now(),
    };
    
    redis.publish("system_broadcasts", serde_json::to_string(&message)?).await?;
    Ok(())
}

// サブスクライバー（受信側）
async fn listen_to_user_updates(redis: &Connection, user_id: Uuid) -> Result<()> {
    let mut pubsub = redis.get_async_connection().await?.into_pubsub();
    let channel = format!("user_updates:{}", user_id);
    pubsub.subscribe(&channel).await?;
    
    let mut stream = pubsub.into_on_message();
    
    while let Some(msg) = stream.next().await {
        let payload: String = msg.get_payload()?;
        let event: UserUpdateEvent = serde_json::from_str(&payload)?;
        
        // イベント処理
        handle_user_update_event(event).await?;
    }
    
    Ok(())
}

// パターンサブスクライブ
async fn listen_to_all_notifications(redis: &Connection) -> Result<()> {
    let mut pubsub = redis.get_async_connection().await?.into_pubsub();
    pubsub.psubscribe("notifications:*").await?; // パターンマッチ
    
    let mut stream = pubsub.into_on_message();
    
    while let Some(msg) = stream.next().await {
        let channel: String = msg.get_channel_name();
        let payload: String = msg.get_payload()?;
        
        println!("Received from {}: {}", channel, payload);
    }
    
    Ok(())
}
```

**メリット**
- 実装が比較的簡単
- リアルタイム性が最高
- パターンマッチングが便利

**デメリット**
- メッセージが揮発性
- 確実な配信保証なし
- 受信者がいないと消失

---

## 最適な組み合わせ戦略

### 🎯 推奨パターン

#### **1. Webアプリケーション**
```rust
// String/JSON: セッション、設定
// Hash: ユーザープロフィール、商品情報
// Pub/Sub: リアルタイム通知
// Streams: 監査ログ
```

#### **2. E-commerce**
```rust
// String: 商品在庫カウンター
// Hash: 商品詳細、ユーザーカート
// Streams: 注文履歴、在庫変動ログ
// Pub/Sub: 在庫切れ通知、価格更新
```

#### **3. IoT・分析システム**
```rust
// Hash: デバイス状態、設定
// Streams: センサーデータ、イベントログ
// Pub/Sub: アラート、リアルタイム監視
// String: 集計結果キャッシュ
```

#### **4. チャット・SNS**
```rust
// Hash: ユーザープロフィール
// Streams: メッセージ履歴、ユーザーアクティビティ
// Pub/Sub: リアルタイムメッセージ配信
// String: オンライン状態、未読数
```

---

## パフォーマンス比較

| 操作 | String/JSON | Hash | Streams | Pub/Sub |
|------|-------------|------|---------|---------|
| **読み取り** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **書き込み** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **部分更新** | ⭐⭐ | ⭐⭐⭐⭐⭐ | ❌ | ❌ |
| **Range検索** | ❌ | ❌ | ⭐⭐⭐⭐⭐ | ❌ |
| **メモリ効率** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## まとめ

### 🚀 **開発速度重視**: String/JSON + Pub/Sub
- プロトタイプ、MVP開発
- Serdeで楽々シリアライゼーション

### ⚡ **パフォーマンス重視**: Hash + Streams
- 本格運用、大規模システム
- メモリ効率と速度の最適化

### 🔄 **リアルタイム重視**: Pub/Sub + Hash
- ライブアプリケーション
- レスポンシブなユーザー体験

### 📊 **分析・ログ重視**: Streams + Hash
- データドリブンな意思決定
- コンプライアンス対応

**最終的には用途に応じて適切に組み合わせることで、Redisの真の力を発揮できます。**