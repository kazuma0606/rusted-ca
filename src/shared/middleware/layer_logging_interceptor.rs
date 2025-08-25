use std::{
    future::Future,
    sync::Arc,
    time::Instant,
};

use chrono::Utc;

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
};

#[derive(Clone)]
pub struct RequestContext {
    pub request_id: RequestId,
    pub user_context: Option<UserContext>,
    pub http_context: HttpContext,
}

impl RequestContext {
    pub fn new(
        request_id: RequestId,
        user_context: Option<UserContext>,
        http_context: HttpContext,
    ) -> Self {
        Self {
            request_id,
            user_context,
            http_context,
        }
    }
}

#[derive(Clone)]
pub struct LayerLoggingInterceptor {
    collect_usecase: Arc<CollectLogUsecase>,
    request_context: Arc<RequestContext>,
}

impl LayerLoggingInterceptor {
    pub fn new(
        collect_usecase: Arc<CollectLogUsecase>,
        request_context: Arc<RequestContext>,
    ) -> Self {
        Self {
            collect_usecase,
            request_context,
        }
    }

    /// UseCase実行のログ収集
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
            operation: Operation::new(usecase_name).unwrap_or_else(|_| Operation::new("unknown").unwrap()),
            metadata: LogMetadata::with_duration(duration_ms),
        };

        // 非同期でログ収集
        let collect_usecase = self.collect_usecase.clone();
        tokio::spawn(async move {
            if let Err(e) = collect_usecase.collect(log_request).await {
                eprintln!("Failed to collect usecase log: {:?}", e);
            }
        });

        result
    }

    /// Repository呼び出しのログ収集
    pub async fn log_repository_call<T, E>(
        &self,
        repository_name: &str,
        method_name: &str,
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

        let level = if result.is_err() { LogLevel::Error } else { LogLevel::Debug };
        let message = format!(
            "Repository: {}::{} - Duration: {}ms",
            repository_name, method_name, duration_ms
        );

        let log_request = CollectLogRequest {
            timestamp,
            level,
            message,
            request_id: self.request_context.request_id.clone(),
            user_context: self.request_context.user_context.clone(),
            http_context: self.request_context.http_context.clone(),
            architecture_layer: ArchitectureLayer::Infrastructure,
            operation: Operation::new(&format!("{}::{}", repository_name, method_name)).unwrap_or_else(|_| Operation::new("unknown").unwrap()),
            metadata: LogMetadata::with_duration(duration_ms),
        };

        // 非同期でログ収集
        let collect_usecase = self.collect_usecase.clone();
        tokio::spawn(async move {
            if let Err(e) = collect_usecase.collect(log_request).await {
                eprintln!("Failed to collect repository log: {:?}", e);
            }
        });

        result
    }

    /// Domain Service実行のログ収集
    pub async fn log_domain_service<T, E>(
        &self,
        service_name: &str,
        method_name: &str,
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

        let level = if result.is_err() { LogLevel::Warn } else { LogLevel::Debug };
        let message = format!(
            "DomainService: {}::{} - Duration: {}ms",
            service_name, method_name, duration_ms
        );

        let log_request = CollectLogRequest {
            timestamp,
            level,
            message,
            request_id: self.request_context.request_id.clone(),
            user_context: self.request_context.user_context.clone(),
            http_context: self.request_context.http_context.clone(),
            architecture_layer: ArchitectureLayer::Domain,
            operation: Operation::new(&format!("{}::{}", service_name, method_name)).unwrap_or_else(|_| Operation::new("unknown").unwrap()),
            metadata: LogMetadata::with_duration(duration_ms),
        };

        // 非同期でログ収集
        let collect_usecase = self.collect_usecase.clone();
        tokio::spawn(async move {
            if let Err(e) = collect_usecase.collect(log_request).await {
                eprintln!("Failed to collect domain service log: {:?}", e);
            }
        });

        result
    }

    /// 一般的なログ記録（メッセージベース）
    pub async fn log_message(
        &self,
        level: LogLevel,
        message: String,
        architecture_layer: ArchitectureLayer,
        operation_name: &str,
    ) {
        let log_request = CollectLogRequest {
            timestamp: Utc::now(),
            level,
            message,
            request_id: self.request_context.request_id.clone(),
            user_context: self.request_context.user_context.clone(),
            http_context: self.request_context.http_context.clone(),
            architecture_layer,
            operation: Operation::new(operation_name).unwrap_or_else(|_| Operation::new("unknown").unwrap()),
            metadata: LogMetadata::empty(),
        };

        // 非同期でログ収集
        let collect_usecase = self.collect_usecase.clone();
        tokio::spawn(async move {
            if let Err(e) = collect_usecase.collect(log_request).await {
                eprintln!("Failed to collect message log: {:?}", e);
            }
        });
    }
}

/// RequestContextを作成するヘルパー
pub struct RequestContextBuilder;

impl RequestContextBuilder {
    pub fn build_for_http_request(
        method: &str,
        path: &str,
        request_id: RequestId,
        user_context: Option<UserContext>,
    ) -> RequestContext {
        let http_context = HttpContext::new(
            method.to_string(),
            path.to_string(),
            0, // status_code は後で設定
            0, // response_time_ms は後で設定
            None,
            None,
        );

        RequestContext::new(request_id, user_context, http_context)
    }

    pub fn build_for_background_task(
        task_name: &str,
        user_context: Option<UserContext>,
    ) -> RequestContext {
        let request_id = RequestId::from(uuid::Uuid::new_v4());

        let http_context = HttpContext::new(
            "BACKGROUND".to_string(),
            task_name.to_string(),
            0,
            0,
            None,
            None,
        );

        RequestContext::new(request_id, user_context, http_context)
    }
}