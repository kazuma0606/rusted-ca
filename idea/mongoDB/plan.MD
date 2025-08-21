# MongoDB構造化ログシステム実装計画

## 概要

Rusted-CA に MongoDB ベースの構造化ログシステムを実装し、既存のメトリクス基盤と統合することで、システム全体の観測性を向上させます。

## アーキテクチャ設計方針

### 基本方針

1. **ログエンティティ化**: ログにIDを付与し、後追い解析・管理機能に対応
2. **層分離の維持**: クリーンアーキテクチャの責務分離を維持
3. **既存システム統合**: メトリクス収集との連携強化
4. **非同期処理**: パフォーマンス影響を最小化
5. **拡張性確保**: 将来の機能追加に対応可能な設計

### 既存システムからの改善点

- **日付別ファイル分割** → **MongoDB集約検索**による高速検索
- **Error/通常ログ分離** → **統合ログ**にレベル・レイヤー情報付与
- **JSONL形式** → **構造化Document**によるリッチな検索・分析

## 🏗️ 層別実装設計

### 1. Domain Layer - ログドメインモデル

```rust
// src/domain/entity/log_entry.rs
// ログエントリはEntity（ID付き、将来の更新に対応）
pub struct LogEntry {
    // 一意識別子：UUID、アプリケーション層で生成
    id: LogId, // Value Object (UUID)
    // タイムスタンプ：必須、UTC固定
    timestamp: DateTime<Utc>,
    // ログレベル：enum で型安全に
    level: LogLevel,
    // メッセージ：人間が読める形式
    message: String,
    // リクエストID：分散トレーシング用
    request_id: RequestId, // Value Object
    // ユーザーコンテキスト：認証情報
    user_context: Option<UserContext>, // Value Object
    // HTTPコンテキスト：リクエスト情報
    http_context: HttpContext, // Value Object
    // アーキテクチャ層：どの層からのログか
    architecture_layer: ArchitectureLayer, // enum
    // 操作名：具体的な処理名
    operation: Operation, // Value Object
    // メタデータ：追加の構造化情報
    metadata: LogMetadata, // Value Object
    
    // 拡張フィールド（将来の解析機能用）
    // タグ：後から追加可能な分類情報
    tags: Vec<String>,
    // 解析ステータス：解析処理の進行状況
    analysis_status: AnalysisStatus, // enum
    // 関連ログ：関連するログエントリへの参照
    related_log_ids: Vec<LogId>,
}

// src/domain/value_object/log_id.rs
// ログの一意識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogId(Uuid);

// src/domain/value_object/analysis_status.rs
// ログ解析の進行状況
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisStatus {
    Pending,    // 未解析
    Processing, // 解析中
    Completed,  // 完了
    Failed,     // 失敗
}

// src/domain/value_object/log_level.rs
// ログレベルをenum で定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,    // 詳細なデバッグ情報
    Info,     // 一般的な情報
    Warn,     // 警告（処理は継続）
    Error,    // エラー（処理に影響）
    Critical, // 致命的（システム停止級）
}

// src/domain/value_object/architecture_layer.rs
// クリーンアーキテクチャの層を表現
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchitectureLayer {
    Presentation,   // HTTP、controller
    Application,    // UseCase、business logic
    Domain,         // Entity、Value Object
    Infrastructure, // Database、external service
}

// src/domain/value_object/http_context.rs
// HTTPリクエスト情報のValue Object
pub struct HttpContext {
    method: HttpMethod,         // GET, POST, etc.
    path: String,              // /api/user/{id}
    status_code: StatusCode,   // 200, 404, 500, etc.
    response_time_ms: u64,     // 処理時間
    user_agent: Option<String>, // ブラウザ情報
    ip_address: Option<String>, // クライアントIP
}

// src/domain/repository/log_repository.rs
// ログリポジトリのインターフェース
pub trait LogRepositoryInterface: Send + Sync {
    async fn store(&self, entry: &LogEntry) -> DomainResult<()>;
    async fn find_by_id(&self, id: &LogId) -> DomainResult<Option<LogEntry>>;
    async fn find_by_criteria(&self, criteria: &LogSearchCriteria) -> DomainResult<Vec<LogEntry>>;
    async fn update_analysis_status(&self, id: &LogId, status: AnalysisStatus) -> DomainResult<()>;
    async fn add_tags(&self, id: &LogId, tags: Vec<String>) -> DomainResult<()>;
}
```

### 2. Application Layer - ログ管理ユースケース

```rust
// src/application/usecases/logging/collect_log_usecase.rs
// ログ収集のユースケース
pub struct CollectLogUsecase {
    // ログ永続化のリポジトリ（インターフェース）
    log_repository: Arc<dyn LogRepositoryInterface>,
    // メトリクス収集との連携
    metrics_collector: Arc<dyn MetricsCollectorInterface>,
    // UUID生成器
    id_generator: Arc<dyn IdGenerator>,
}

impl CollectLogUsecase {
    // ログエントリを収集・永続化
    pub async fn collect(&self, request: CollectLogRequest) -> ApplicationResult<LogId> {
        // 1. ログIDを生成
        let log_id = self.id_generator.generate_log_id();
        
        // 2. ドメインエンティティを構築
        let log_entry = LogEntry::new(
            log_id,
            request.timestamp,
            request.level,
            request.message,
            request.request_id,
            request.user_context,
            request.http_context,
            request.architecture_layer,
            request.operation,
            request.metadata,
        )?;
        
        // 3. ドメインルールに基づくバリデーション
        // 4. ログリポジトリへの永続化
        self.log_repository.store(&log_entry).await?;
        
        // 5. メトリクス更新（エラー率、レスポンス時間など）
        self.metrics_collector.record_log_event(&log_entry).await?;
        
        // 6. 必要に応じてアラート通知
        if log_entry.level == LogLevel::Critical {
            // Discord通知など
        }
        
        Ok(log_id)
    }
}

// src/application/usecases/logging/search_logs_usecase.rs
// ログ検索のユースケース
pub struct SearchLogsUsecase {
    log_repository: Arc<dyn LogRepositoryInterface>,
}

impl SearchLogsUsecase {
    // 条件に基づくログ検索
    pub async fn search(&self, criteria: LogSearchCriteria) -> ApplicationResult<Vec<LogEntry>> {
        // 1. 検索条件のバリデーション
        criteria.validate()?;
        
        // 2. リポジトリでの検索実行
        let logs = self.log_repository.find_by_criteria(&criteria).await?;
        
        // 3. 結果の集約・フィルタリング
        Ok(logs)
    }

    // パフォーマンス分析用の集約クエリ
    pub async fn analyze_performance(&self, criteria: AnalysisCriteria) -> ApplicationResult<PerformanceReport> {
        // 1. レスポンス時間の統計情報
        // 2. エラー率の計算
        // 3. レイヤー別の性能分析
        
        let logs = self.log_repository.find_by_criteria(&criteria.to_search_criteria()).await?;
        
        let report = PerformanceReport::analyze(logs)?;
        Ok(report)
    }
}

// src/application/usecases/logging/manage_log_usecase.rs
// ログ管理のユースケース（解析、タグ付け等）
pub struct ManageLogUsecase {
    log_repository: Arc<dyn LogRepositoryInterface>,
}

impl ManageLogUsecase {
    // ログの解析ステータス更新
    pub async fn update_analysis_status(&self, id: LogId, status: AnalysisStatus) -> ApplicationResult<()> {
        self.log_repository.update_analysis_status(&id, status).await?;
        Ok(())
    }
    
    // ログへのタグ追加
    pub async fn add_tags(&self, id: LogId, tags: Vec<String>) -> ApplicationResult<()> {
        self.log_repository.add_tags(&id, tags).await?;
        Ok(())
    }
}

```

### 3. Infrastructure Layer - MongoDB統合

```rust
// src/infrastructure/logging/mongodb_log_repository.rs
// MongoDBを使ったログリポジトリの実装
pub struct MongoDbLogRepository {
    // MongoDB接続
    collection: Collection<LogEntryDocument>, // MongoDB用のDocument型
    // 非同期バッファ（パフォーマンス最適化）
    buffer: Arc<Mutex<Vec<LogEntry>>>,
    // バックグラウンドタスク制御
    flush_handle: Option<JoinHandle<()>>,
    // バッファ設定
    buffer_config: BufferConfig,
}

impl MongoDbLogRepository {
    pub async fn new(
        client: Client, 
        database_name: &str, 
        collection_name: &str
    ) -> InfrastructureResult<Self> {
        let db = client.database(database_name);
        let collection = db.collection::<LogEntryDocument>(collection_name);
        
        // インデックス作成
        Self::create_indexes(&collection).await?;
        
        let repository = Self {
            collection,
            buffer: Arc::new(Mutex::new(Vec::new())),
            flush_handle: None,
            buffer_config: BufferConfig::default(),
        };
        
        // バックグラウンド書き込みタスク開始
        repository.start_background_flush().await;
        
        Ok(repository)
    }
    
    async fn create_indexes(collection: &Collection<LogEntryDocument>) -> InfrastructureResult<()> {
        // 検索性能向上のためのインデックス
        let indexes = vec![
            IndexModel::builder()
                .keys(doc! { "timestamp": -1 })
                .options(IndexOptions::builder().name("timestamp_desc".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "level": 1, "architecture_layer": 1 })
                .options(IndexOptions::builder().name("level_layer".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "request_id": 1 })
                .options(IndexOptions::builder().name("request_id".to_string()).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "log_id": 1 })
                .options(IndexOptions::builder().name("log_id_unique".to_string()).unique(true).build())
                .build(),
        ];
        
        collection.create_indexes(indexes, None).await?;
        Ok(())
    }
}

impl LogRepositoryInterface for MongoDbLogRepository {
    // ログエントリの永続化（非同期バッファリング）
    async fn store(&self, entry: &LogEntry) -> DomainResult<()> {
        // 1. バッファに追加（高速レスポンス）
        {
            let mut buffer = self.buffer.lock().await;
            buffer.push(entry.clone());
        }
        
        // 2. バッファサイズチェック（即座にフラッシュが必要か）
        let buffer_size = {
            let buffer = self.buffer.lock().await;
            buffer.len()
        };
        
        if buffer_size >= self.buffer_config.max_buffer_size {
            self.flush_buffer().await?;
        }
        
        Ok(())
    }
    
    async fn find_by_id(&self, id: &LogId) -> DomainResult<Option<LogEntry>> {
        let filter = doc! { "log_id": id.to_string() };
        
        match self.collection.find_one(filter, None).await? {
            Some(doc) => Ok(Some(doc.try_into()?)),
            None => Ok(None),
        }
    }

    // 検索機能（MongoDB aggregation使用）
    async fn find_by_criteria(&self, criteria: &LogSearchCriteria) -> DomainResult<Vec<LogEntry>> {
        // 1. 検索条件 → MongoDB query変換
        let filter = criteria.to_mongodb_filter();
        let options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(criteria.limit.unwrap_or(100) as i64)
            .skip(criteria.offset.unwrap_or(0) as u64)
            .build();
        
        // 2. インデックスを活用した高速検索
        let mut cursor = self.collection.find(filter, options).await?;
        let mut logs = Vec::new();
        
        // 3. Document → ドメインオブジェクト変換
        while let Some(doc) = cursor.try_next().await? {
            logs.push(doc.try_into()?);
        }
        
        Ok(logs)
    }
    
    async fn update_analysis_status(&self, id: &LogId, status: AnalysisStatus) -> DomainResult<()> {
        let filter = doc! { "log_id": id.to_string() };
        let update = doc! { 
            "$set": { 
                "analysis_status": status.to_string(),
                "updated_at": Utc::now() 
            } 
        };
        
        self.collection.update_one(filter, update, None).await?;
        Ok(())
    }
    
    async fn add_tags(&self, id: &LogId, tags: Vec<String>) -> DomainResult<()> {
        let filter = doc! { "log_id": id.to_string() };
        let update = doc! { 
            "$addToSet": { "tags": { "$each": tags } },
            "$set": { "updated_at": Utc::now() }
        };
        
        self.collection.update_one(filter, update, None).await?;
        Ok(())
    }
}

// src/infrastructure/logging/log_entry_document.rs
// MongoDB用のDocument型（インフラ層のみで使用）
#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntryDocument {
    // MongoDB用の_id（物理ID、自動生成）
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub mongo_id: Option<ObjectId>,
    
    // ドメインオブジェクトのフィールド
    pub log_id: String,                    // LogId(UUID)をStringに変換
    pub timestamp: DateTime<Utc>,
    pub level: String,                     // LogLevelをStringに変換
    pub message: String,
    pub request_id: String,                // RequestIdをStringに変換
    pub architecture_layer: String,        // ArchitectureLayerをStringに変換
    pub operation: String,
    
    // オプションフィールド
    pub user_context: Option<UserContextDocument>,
    pub http_context: HttpContextDocument,
    pub metadata: serde_json::Value,
    
    // 拡張フィールド
    pub tags: Vec<String>,
    pub analysis_status: String,           // AnalysisStatusをStringに変換
    pub related_log_ids: Vec<String>,
    
    // インデックス用の追加フィールド
    pub indexed_at: DateTime<Utc>,         // 検索最適化
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpContextDocument {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserContextDocument {
    pub user_id: String,
    pub username: Option<String>,
    pub role: Option<String>,
}

// ドメインオブジェクト ↔ MongoDB Document の変換
impl From<LogEntry> for LogEntryDocument {
    fn from(entry: LogEntry) -> Self {
        Self {
            mongo_id: None, // MongoDBが自動生成
            log_id: entry.id().to_string(),
            timestamp: entry.timestamp(),
            level: entry.level().to_string(),
            message: entry.message().clone(),
            request_id: entry.request_id().to_string(),
            architecture_layer: entry.architecture_layer().to_string(),
            operation: entry.operation().to_string(),
            user_context: entry.user_context().map(Into::into),
            http_context: entry.http_context().into(),
            metadata: entry.metadata().to_json(),
            tags: entry.tags().clone(),
            analysis_status: entry.analysis_status().to_string(),
            related_log_ids: entry.related_log_ids().iter().map(|id| id.to_string()).collect(),
            indexed_at: Utc::now(),
            created_at: entry.timestamp(),
            updated_at: Utc::now(),
        }
    }
}

impl TryFrom<LogEntryDocument> for LogEntry {
    type Error = InfrastructureError;
    
    fn try_from(doc: LogEntryDocument) -> Result<Self, Self::Error> {
        // Document → ドメインオブジェクトの逆変換
        // バリデーション、型変換等を実施
        LogEntry::reconstruct(
            LogId::from_string(&doc.log_id)?,
            doc.timestamp,
            LogLevel::from_string(&doc.level)?,
            doc.message,
            RequestId::from_string(&doc.request_id)?,
            doc.user_context.map(TryInto::try_into).transpose()?,
            doc.http_context.try_into()?,
            ArchitectureLayer::from_string(&doc.architecture_layer)?,
            Operation::from_string(&doc.operation)?,
            LogMetadata::from_json(doc.metadata)?,
            doc.tags,
            AnalysisStatus::from_string(&doc.analysis_status)?,
            doc.related_log_ids.into_iter()
                .map(|id| LogId::from_string(&id))
                .collect::<Result<Vec<_>, _>>()?,
        )
    }
}

```

### 4. Presentation Layer - 自動ログ収集

```rust
// src/shared/middleware/logging_middleware.rs
// Axum Tower Middleware としてログ収集を実装
#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    // 内部サービス
    inner: S,
    // ログ収集ユースケース
    collect_usecase: Arc<CollectLogUsecase>,
    // リクエストID生成器
    request_id_generator: Arc<dyn RequestIdGenerator>,
}

impl<S> LoggingMiddleware<S> {
    pub fn new(
        inner: S, 
        collect_usecase: Arc<CollectLogUsecase>,
        request_id_generator: Arc<dyn RequestIdGenerator>
    ) -> Self {
        Self {
            inner,
            collect_usecase,
            request_id_generator,
        }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LoggingMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // 1. リクエスト開始時点の情報収集
        let start_time = Instant::now();
        let timestamp = Utc::now();
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let user_agent = req.headers()
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        // リクエストID生成（ヘッダーから取得 or 新規生成）
        let request_id = req.headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| RequestId::from_string(s).ok())
            .unwrap_or_else(|| self.request_id_generator.generate());

        // ユーザーコンテキスト抽出（JWTから）
        let user_context = extract_user_context_from_headers(req.headers());
        
        let collect_usecase = self.collect_usecase.clone();
        let mut inner = self.inner.clone();
        
        Box::pin(async move {
            // 2. 内部サービス呼び出し
            let response = inner.call(req).await?;
            
            // 3. レスポンス完了時点の情報収集
            let end_time = Instant::now();
            let response_time_ms = end_time.duration_since(start_time).as_millis() as u64;
            let status_code = response.status();
            
            // 4. LogEntry作成してユースケース呼び出し（非同期で実行）
            let log_request = CollectLogRequest {
                timestamp,
                level: determine_log_level_from_status(status_code),
                message: format!("{} {} - {}", method, path, status_code),
                request_id: request_id.clone(),
                user_context,
                http_context: HttpContext::new(
                    method.into(),
                    path,
                    status_code.as_u16(),
                    response_time_ms,
                    user_agent,
                    None, // IP address - extract from connection info if needed
                ),
                architecture_layer: ArchitectureLayer::Presentation,
                operation: Operation::new(&format!("HTTP_{}", method)),
                metadata: LogMetadata::empty(),
            };
            
            // 非同期でログ収集（レスポンス遅延なし）
            tokio::spawn(async move {
                if let Err(e) = collect_usecase.collect(log_request).await {
                    // ログ収集エラーは別途記録（サービス継続）
                    eprintln!("Failed to collect log: {:?}", e);
                }
            });
            
            Ok(response)
        })
    }
}

// ヘルパー関数
fn determine_log_level_from_status(status: StatusCode) -> LogLevel {
    match status.as_u16() {
        200..=299 => LogLevel::Info,
        300..=399 => LogLevel::Info,
        400..=499 => LogLevel::Warn,
        500..=599 => LogLevel::Error,
        _ => LogLevel::Debug,
    }
}

fn extract_user_context_from_headers(headers: &HeaderMap) -> Option<UserContext> {
    // JWT トークンからユーザー情報を抽出
    // 実装は認証システムに依存
    None
}

// src/shared/middleware/layer_logging_interceptor.rs
// アーキテクチャ層ごとのログ収集
pub struct LayerLoggingInterceptor {
    collect_usecase: Arc<CollectLogUsecase>,
    request_context: Arc<RequestContext>, // リクエスト固有の情報
}

impl LayerLoggingInterceptor {
    // UseCase実行のログ収集
    pub async fn log_usecase_execution<T, E>(
        &self,
        usecase_name: &str,
        operation: impl Future<Output = Result<T, E>>,
    ) -> Result<T, E>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let start_time = Instant::now();
        let timestamp = Utc::now();
        
        let result = operation.await;
        
        let end_time = Instant::now();
        let duration_ms = end_time.duration_since(start_time).as_millis() as u64;
        
        let level = if result.is_err() { LogLevel::Error } else { LogLevel::Info };
        let message = format!("UseCase: {} - Duration: {}ms", usecase_name, duration_ms);
        
        let log_request = CollectLogRequest {
            timestamp,
            level,
            message,
            request_id: self.request_context.request_id.clone(),
            user_context: self.request_context.user_context.clone(),
            http_context: self.request_context.http_context.clone(),
            architecture_layer: ArchitectureLayer::Application,
            operation: Operation::new(usecase_name),
            metadata: LogMetadata::with_duration(duration_ms),
        };
        
        // 非同期でログ収集
        let collect_usecase = self.collect_usecase.clone();
        tokio::spawn(async move {
            let _ = collect_usecase.collect(log_request).await;
        });
        
        result
    }
    
    // Repository呼び出しのログ収集
    pub async fn log_repository_call<T, E>(
        &self,
        repository_name: &str,
        method_name: &str,
        operation: impl Future<Output = Result<T, E>>,
    ) -> Result<T, E>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        // 同様の実装...
        operation.await
    }
}

```

### 5. DI Container統合

```rust
// src/infrastructure/di/container.rs (既存ファイルへの追加)
pub struct DIContainer {
    // 既存のフィールド...
    pub tidb_pool: MySqlPool,
    pub redis_pool: RedisPool,
    
    // 新規追加：MongoDB関連
    pub mongodb_client: Client,
    pub log_repository: Arc<dyn LogRepositoryInterface>,
    
    // 新規追加：ログ関連ユースケース
    pub collect_log_usecase: Arc<CollectLogUsecase>,
    pub search_logs_usecase: Arc<SearchLogsUsecase>,
    pub manage_log_usecase: Arc<ManageLogUsecase>,
    
    // 既存フィールド（メトリクス、その他のユースケース等）
    // ...
}

impl DIContainer {
    pub async fn new(tidb_pool: MySqlPool, redis_pool: RedisPool) -> InfrastructureResult<Self> {
        // 1. MongoDB接続初期化
        let mongodb_client = Client::with_uri_str("mongodb://admin:admin123@localhost:27017/")
            .await
            .map_err(|e| InfrastructureError::DatabaseConnection(e.to_string()))?;
        
        // 2. ログリポジトリ初期化
        let log_repository = Arc::new(
            MongoDbLogRepository::new(
                mongodb_client.clone(),
                "rusted_ca_logs",
                "log_entries"
            ).await?
        ) as Arc<dyn LogRepositoryInterface>;

        // 3. ID生成器初期化
        let id_generator = Arc::new(UuidGenerator::new()) as Arc<dyn IdGenerator>;

        // 4. 既存のメトリクス収集器（既に初期化されている想定）
        let metrics_collector = Arc::new(MetricsCollector::new()) as Arc<dyn MetricsCollectorInterface>;

        // 5. ユースケース初期化
        let collect_log_usecase = Arc::new(CollectLogUsecase::new(
            log_repository.clone(),
            metrics_collector.clone(),
            id_generator.clone(),
        ));

        let search_logs_usecase = Arc::new(SearchLogsUsecase::new(
            log_repository.clone(),
        ));

        let manage_log_usecase = Arc::new(ManageLogUsecase::new(
            log_repository.clone(),
        ));

        // 6. 既存のユースケース初期化（ユーザー管理等）
        // ...

        Ok(Self {
            tidb_pool,
            redis_pool,
            mongodb_client,
            log_repository,
            collect_log_usecase,
            search_logs_usecase,
            manage_log_usecase,
            // 既存フィールドの初期化...
        })
    }
}
```

### 6. Router統合

```rust
// src/infrastructure/web/api_router.rs (既存ファイルへの追加)
pub fn build_api_router(di: Arc<DIContainer>) -> Router {
    Router::new()
        // 既存API
        .route("/api/user", post(create_user_handler))
        .route("/api/user/:id", get(get_user_handler))
        .route("/api/user/:id", put(update_user_handler))
        .route("/api/user/:id", delete(delete_user_handler))
        .route("/health", get(health_check_handler))
        
        // ログ管理API追加
        .route("/api/logs/search", post(search_logs_handler))
        .route("/api/logs/analyze", get(analyze_performance_handler))
        .route("/api/logs/:id", get(get_log_handler))
        .route("/api/logs/:id/tags", post(add_tags_handler))
        .route("/api/logs/:id/status", put(update_analysis_status_handler))
        
        // ミドルウェアスタック（順序重要）
        .layer(LoggingLayer::new(
            di.collect_log_usecase.clone(),
            di.id_generator.clone()
        ))  // ←新規追加（最外層）
        .layer(MetricsLayer::new(di.metrics_collector.clone()))    // ←既存
        .layer(SecurityHeadersLayer::new())                        // ←既存
        .layer(CorsLayer::new())                                   // ←既存
        .with_state(di)
}

// ログ検索API
async fn search_logs_handler(
    State(di): State<Arc<DIContainer>>,
    Json(request): Json<LogSearchRequest>,
) -> PresentationResult<Json<LogSearchResponse>> {
    // 1. リクエスト → ドメインオブジェクト変換
    let criteria = LogSearchCriteria::from(request);
    
    // 2. ユースケース実行
    let logs = di.search_logs_usecase.search(criteria).await
        .map_err(PresentationError::from)?;
    
    // 3. 結果 → レスポンス変換
    let response = LogSearchResponse::from(logs);
    
    Ok(Json(response))
}

// ログ分析API
async fn analyze_performance_handler(
    State(di): State<Arc<DIContainer>>,
    Query(params): Query<AnalysisParams>,
) -> PresentationResult<Json<PerformanceReportResponse>> {
    let criteria = AnalysisCriteria::from(params);
    
    let report = di.search_logs_usecase.analyze_performance(criteria).await
        .map_err(PresentationError::from)?;
    
    let response = PerformanceReportResponse::from(report);
    
    Ok(Json(response))
}

// 個別ログ取得API
async fn get_log_handler(
    State(di): State<Arc<DIContainer>>,
    Path(log_id): Path<String>,
) -> PresentationResult<Json<LogResponse>> {
    let log_id = LogId::from_string(&log_id)
        .map_err(|e| PresentationError::InvalidRequest(e.to_string()))?;
    
    match di.log_repository.find_by_id(&log_id).await
        .map_err(PresentationError::from)?
    {
        Some(log) => Ok(Json(LogResponse::from(log))),
        None => Err(PresentationError::NotFound("Log not found".to_string())),
    }
}

// タグ追加API
async fn add_tags_handler(
    State(di): State<Arc<DIContainer>>,
    Path(log_id): Path<String>,
    Json(request): Json<AddTagsRequest>,
) -> PresentationResult<Json<ApiResponse<()>>> {
    let log_id = LogId::from_string(&log_id)
        .map_err(|e| PresentationError::InvalidRequest(e.to_string()))?;
    
    di.manage_log_usecase.add_tags(log_id, request.tags).await
        .map_err(PresentationError::from)?;
    
    Ok(Json(ApiResponse::success(())))
}
```

## 🎯 実装の方針・特徴

### ✅ ID付きエンティティ設計
- ログにUUID識別子を付与
- 将来の解析・管理機能に対応
- MongoDB物理IDとの明確な分離

### ✅ クリーンアーキテクチャ準拠
- ログエントリはドメインEntity
- 各層の責務を明確に分離
- 依存関係の方向を適切に保持

### ✅ 既存システムとの統合
- メトリクス収集との連携強化
- エラーハンドリング体系の活用
- DI Containerでの一元管理

### ✅ パフォーマンス最適化
- 非同期バッファリング
- MongoDB aggregationの活用
- ミドルウェアでの自動収集
- インデックス最適化

### ✅ 運用・拡張性
- 構造化ログによる高速検索
- リクエストトレーシング
- レイヤー横断的な可観測性
- タグ付け・解析ステータス管理

## 📋 実装フェーズ

**Phase 1**: ドメインモデル・リポジトリインターフェース  
**Phase 2**: MongoDB統合・基本ログ収集  
**Phase 3**: ミドルウェア統合・自動ログ収集  
**Phase 4**: 検索・分析API・管理機能

この設計により、既存システムを壊すことなく包括的なログ収集システムを構築できます！