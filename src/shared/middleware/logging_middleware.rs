use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::Response,
};
use chrono::Utc;
use tower::{Layer, Service};

use crate::{
    application::usecases::logging::collect_log_usecase::CollectLogUsecase,
    application::dto::collect_log_request::CollectLogRequest,
    domain::value_object::{
        log_level::LogLevel,
        architecture_layer::ArchitectureLayer,
        operation::Operation,
        log_metadata::LogMetadata,
        request_id::RequestId,
        user_context::UserContext,
        http_context::HttpContext,
    },
    shared::utils::uuid_generator::UuidGenerator,
};

#[derive(Clone)]
pub struct LoggingLayer {
    collect_usecase: Arc<CollectLogUsecase>,
    request_id_generator: Arc<UuidGenerator>,
}

impl LoggingLayer {
    pub fn new(
        collect_usecase: Arc<CollectLogUsecase>,
        request_id_generator: Arc<UuidGenerator>,
    ) -> Self {
        Self {
            collect_usecase,
            request_id_generator,
        }
    }
}

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddleware {
            inner,
            collect_usecase: self.collect_usecase.clone(),
            request_id_generator: self.request_id_generator.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
    collect_usecase: Arc<CollectLogUsecase>,
    request_id_generator: Arc<UuidGenerator>,
}

impl<S> Service<Request> for LoggingMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        // リクエスト開始時点の情報収集
        let start_time = Instant::now();
        let timestamp = Utc::now();
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let user_agent = req.headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // リクエストID生成（ヘッダーから取得 or 新規生成）
        let request_id = req.headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| RequestId::from_string(s).ok())
            .unwrap_or_else(|| self.request_id_generator.generate_request_id());

        // ユーザーコンテキスト抽出（JWTから）
        let user_context = extract_user_context_from_headers(req.headers());

        // IP address extraction from connection info
        let ip_address = req.headers()
            .get("x-forwarded-for")
            .or_else(|| req.headers().get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let collect_usecase = self.collect_usecase.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // 内部サービス呼び出し
            let response = inner.call(req).await?;

            // レスポンス完了時点の情報収集
            let end_time = Instant::now();
            let response_time_ms = end_time.duration_since(start_time).as_millis() as u64;
            let status_code = response.status();

            // LogEntry作成してユースケース呼び出し（非同期で実行）
            let log_request = CollectLogRequest {
                timestamp,
                level: determine_log_level_from_status(status_code),
                message: format!("{} {} - {}", method, path, status_code),
                request_id: request_id.clone(),
                user_context,
                http_context: HttpContext::new(
                    method.as_str().to_string(),
                    path,
                    status_code.as_u16(),
                    response_time_ms,
                    user_agent,
                    ip_address,
                ),
                architecture_layer: ArchitectureLayer::Presentation,
                operation: Operation::new(&format!("HTTP_{}", method)).unwrap_or_else(|_| Operation::new("HTTP_UNKNOWN").unwrap()),
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
    // 実装は認証システムに依存するため、現在は None を返す
    // 将来的には JWT デコードしてユーザー情報を抽出
    let auth_header = headers.get("authorization")?;
    let token = auth_header.to_str().ok()?.strip_prefix("Bearer ")?;
    
    // 簡単な実装例（実際はJWTデコードが必要）
    if !token.is_empty() {
        // TODO: JWT デコードしてユーザー情報を抽出
        None
    } else {
        None
    }
}