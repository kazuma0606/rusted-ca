use std::{sync::Arc, time::Instant};

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post, put},
};
use chrono::Utc;

use crate::{
    application::usecases::logging::{
        collect_log_usecase::CollectLogUsecase, manage_log_usecase::ManageLogUsecase,
        search_logs_usecase::SearchLogsUsecase,
    },
    domain::{
        repository::log_repository::LogSearchCriteria,
        value_object::{
            analysis_status::AnalysisStatus, architecture_layer::ArchitectureLayer, log_id::LogId,
            log_level::LogLevel,
        },
    },
    presentation::dto::{log_search_request::*, log_search_response::*},
};

pub struct LogController {
    search_logs_usecase: Arc<SearchLogsUsecase>,
    manage_log_usecase: Arc<ManageLogUsecase>,
    collect_log_usecase: Arc<CollectLogUsecase>,
}

impl LogController {
    pub fn new(
        search_logs_usecase: Arc<SearchLogsUsecase>,
        manage_log_usecase: Arc<ManageLogUsecase>,
        collect_log_usecase: Arc<CollectLogUsecase>,
    ) -> Self {
        Self {
            search_logs_usecase,
            manage_log_usecase,
            collect_log_usecase,
        }
    }

    // ログ検索エンドポイント
    pub async fn search_logs(
        State(controller): State<Arc<LogController>>,
        Query(request): Query<LogSearchRequest>,
    ) -> Result<Json<ApiResponse<LogSearchResponse>>, StatusCode> {
        let start_time = Instant::now();

        // リクエストをドメインオブジェクトに変換
        let criteria = match Self::convert_to_search_criteria(&request) {
            Ok(criteria) => criteria,
            Err(e) => {
                return Ok(Json(ApiResponse::error(format!(
                    "Invalid search criteria: {}",
                    e
                ))));
            }
        };

        // ログ検索実行
        match controller
            .search_logs_usecase
            .search(criteria.clone())
            .await
        {
            Ok(search_result) => {
                let query_time_ms = start_time.elapsed().as_millis() as u64;

                let response = LogSearchResponse {
                    logs: search_result
                        .logs
                        .into_iter()
                        .map(|entry| Self::convert_to_log_entry_response(entry))
                        .collect(),
                    total_count: search_result.total_count.unwrap_or(0),
                    has_more: search_result.has_more,
                    search_metadata: SearchMetadata {
                        query_time_ms,
                        search_criteria: format!("{:?}", criteria),
                        filters_applied: Self::get_applied_filters(&request),
                    },
                };

                Ok(Json(ApiResponse::success(response)))
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Search failed: {}", e)))),
        }
    }

    // 特定ログ取得エンドポイント
    pub async fn get_log_by_id(
        State(controller): State<Arc<LogController>>,
        Path(log_id): Path<String>,
    ) -> Result<Json<ApiResponse<LogEntryResponse>>, StatusCode> {
        let log_id = match LogId::from_string(&log_id) {
            Ok(id) => id,
            Err(e) => return Ok(Json(ApiResponse::error(format!("Invalid log ID: {}", e)))),
        };

        match controller.search_logs_usecase.find_by_id(&log_id).await {
            Ok(Some(entry)) => {
                let response = Self::convert_to_log_entry_response(entry);
                Ok(Json(ApiResponse::success(response)))
            }
            Ok(None) => Ok(Json(ApiResponse::error("Log not found".to_string()))),
            Err(e) => Ok(Json(ApiResponse::error(format!(
                "Failed to retrieve log: {}",
                e
            )))),
        }
    }

    // リクエストIDによるログ取得（トレーシング用）
    pub async fn get_logs_by_request_id(
        State(controller): State<Arc<LogController>>,
        Path(request_id): Path<String>,
    ) -> Result<Json<ApiResponse<Vec<LogEntryResponse>>>, StatusCode> {
        let request_id =
            match crate::domain::value_object::request_id::RequestId::from_string(&request_id) {
                Ok(id) => id,
                Err(e) => {
                    return Ok(Json(ApiResponse::error(format!(
                        "Invalid request ID: {}",
                        e
                    ))));
                }
            };

        match controller
            .search_logs_usecase
            .find_by_request_id(&request_id)
            .await
        {
            Ok(logs) => {
                let response: Vec<LogEntryResponse> = logs
                    .into_iter()
                    .map(|entry| Self::convert_to_log_entry_response(entry))
                    .collect();
                Ok(Json(ApiResponse::success(response)))
            }
            Err(e) => Ok(Json(ApiResponse::error(format!(
                "Failed to retrieve logs: {}",
                e
            )))),
        }
    }

    // パフォーマンスメトリクス取得
    pub async fn get_performance_metrics(
        State(controller): State<Arc<LogController>>,
        Query(request): Query<LogPerformanceMetricsRequest>,
    ) -> Result<Json<ApiResponse<LogPerformanceMetricsResponse>>, StatusCode> {
        match controller
            .search_logs_usecase
            .get_performance_metrics(request.start_time, request.end_time)
            .await
        {
            Ok(metrics) => {
                let response = LogPerformanceMetricsResponse {
                    total_requests: metrics.total_requests,
                    min_response_time_ms: metrics.min_response_time_ms,
                    max_response_time_ms: metrics.max_response_time_ms,
                    avg_response_time_ms: metrics.avg_response_time_ms,
                    p95_response_time_ms: metrics.p95_response_time_ms,
                    error_rate: metrics.error_rate,
                    requests_by_status_code: metrics.requests_by_status_code,
                    requests_by_layer: metrics
                        .requests_by_layer
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v))
                        .collect(),
                    time_range: TimeRangeResponse {
                        start: request.start_time,
                        end: request.end_time,
                    },
                };
                Ok(Json(ApiResponse::success(response)))
            }
            Err(e) => Ok(Json(ApiResponse::error(format!(
                "Failed to get metrics: {}",
                e
            )))),
        }
    }

    // ログ削除（クリーンアップ）
    pub async fn cleanup_logs(
        State(controller): State<Arc<LogController>>,
        Json(request): Json<LogCleanupRequest>,
    ) -> Result<Json<ApiResponse<LogCleanupResponse>>, StatusCode> {
        let start_time = Instant::now();
        let dry_run = request.dry_run.unwrap_or(false);

        let result = if dry_run {
            // ドライラン：削除対象数のみ取得
            match controller
                .manage_log_usecase
                .count_logs_older_than(request.cutoff_time)
                .await
            {
                Ok(count) => Ok(count),
                Err(e) => Err(e),
            }
        } else {
            // 実際の削除実行
            controller
                .manage_log_usecase
                .delete_logs_older_than(request.cutoff_time)
                .await
        };

        match result {
            Ok(deleted_count) => {
                let execution_time_ms = start_time.elapsed().as_millis() as u64;
                let response = LogCleanupResponse {
                    deleted_count,
                    dry_run,
                    execution_time_ms,
                    cutoff_time: request.cutoff_time,
                };
                Ok(Json(ApiResponse::success(response)))
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Cleanup failed: {}", e)))),
        }
    }

    // タグ管理エンドポイント
    pub async fn add_tags(
        State(controller): State<Arc<LogController>>,
        Json(request): Json<AddTagsRequest>,
    ) -> Result<Json<ApiResponse<String>>, StatusCode> {
        let log_id = match LogId::from_string(&request.log_id) {
            Ok(id) => id,
            Err(e) => return Ok(Json(ApiResponse::error(format!("Invalid log ID: {}", e)))),
        };

        match controller
            .manage_log_usecase
            .add_tags(&log_id, request.tags)
            .await
        {
            Ok(_) => Ok(Json(ApiResponse::success(
                "Tags added successfully".to_string(),
            ))),
            Err(e) => Ok(Json(ApiResponse::error(format!(
                "Failed to add tags: {}",
                e
            )))),
        }
    }

    pub async fn remove_tag(
        State(controller): State<Arc<LogController>>,
        Json(request): Json<RemoveTagRequest>,
    ) -> Result<Json<ApiResponse<String>>, StatusCode> {
        let log_id = match LogId::from_string(&request.log_id) {
            Ok(id) => id,
            Err(e) => return Ok(Json(ApiResponse::error(format!("Invalid log ID: {}", e)))),
        };

        match controller
            .manage_log_usecase
            .remove_tag(&log_id, &request.tag)
            .await
        {
            Ok(_) => Ok(Json(ApiResponse::success(
                "Tag removed successfully".to_string(),
            ))),
            Err(e) => Ok(Json(ApiResponse::error(format!(
                "Failed to remove tag: {}",
                e
            )))),
        }
    }

    // 分析ステータス更新
    pub async fn update_analysis_status(
        State(controller): State<Arc<LogController>>,
        Json(request): Json<UpdateAnalysisStatusRequest>,
    ) -> Result<Json<ApiResponse<String>>, StatusCode> {
        let log_id = match LogId::from_string(&request.log_id) {
            Ok(id) => id,
            Err(e) => return Ok(Json(ApiResponse::error(format!("Invalid log ID: {}", e)))),
        };

        let status = match AnalysisStatus::from_string(&request.status) {
            Ok(status) => status,
            Err(e) => return Ok(Json(ApiResponse::error(format!("Invalid status: {}", e)))),
        };

        match controller
            .manage_log_usecase
            .update_analysis_status(&log_id, status)
            .await
        {
            Ok(_) => Ok(Json(ApiResponse::success(
                "Status updated successfully".to_string(),
            ))),
            Err(e) => Ok(Json(ApiResponse::error(format!(
                "Failed to update status: {}",
                e
            )))),
        }
    }

    // ログ管理ダッシュボード（HTML）
    pub async fn log_dashboard() -> Html<String> {
        let html = include_str!("../../../templates/log_dashboard.html");
        Html(html.to_string())
    }

    // ログ集約データ（ダッシュボード用）
    pub async fn get_log_aggregation(
        State(controller): State<Arc<LogController>>,
    ) -> Result<Json<ApiResponse<LogAggregationResponse>>, StatusCode> {
        // 24時間以内のデータを集約
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::hours(24);

        match controller
            .search_logs_usecase
            .get_aggregated_data(start_time, end_time)
            .await
        {
            Ok(aggregation) => Ok(Json(ApiResponse::success(aggregation))),
            Err(e) => Ok(Json(ApiResponse::error(format!(
                "Failed to get aggregation: {}",
                e
            )))),
        }
    }

    // ヘルパーメソッド
    fn convert_to_search_criteria(request: &LogSearchRequest) -> Result<LogSearchCriteria, String> {
        let level = if let Some(ref level_str) = request.level {
            Some(LogLevel::from_string(level_str).map_err(|e| e.to_string())?)
        } else {
            None
        };

        let architecture_layer = if let Some(ref layer_str) = request.architecture_layer {
            Some(ArchitectureLayer::from_string(layer_str).map_err(|e| e.to_string())?)
        } else {
            None
        };

        let request_id = if let Some(ref req_id) = request.request_id {
            Some(
                crate::domain::value_object::request_id::RequestId::from_string(req_id)
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };

        let analysis_status = if let Some(ref status_str) = request.analysis_status {
            Some(AnalysisStatus::from_string(status_str).map_err(|e| e.to_string())?)
        } else {
            None
        };

        Ok(LogSearchCriteria {
            level,
            architecture_layer,
            request_id,
            user_id: request.user_id.clone(),
            start_time: request.start_time,
            end_time: request.end_time,
            message_contains: request.message_contains.clone(),
            tags: request.tags.clone(),
            analysis_status,
            limit: request.limit,
            offset: request.offset,
        })
    }

    fn convert_to_log_entry_response(
        entry: crate::domain::entity::log_entry::LogEntry,
    ) -> LogEntryResponse {
        LogEntryResponse {
            log_id: entry.id().to_string(),
            timestamp: entry.timestamp(),
            level: entry.level().to_string(),
            message: entry.message().to_string(),
            request_id: entry.request_id().to_string(),
            user_context: entry.user_context().clone().map(|ctx| UserContextResponse {
                user_id: ctx.user_id().to_string(),
                username: ctx.username().map(|s| s.to_string()),
                role: ctx.role().map(|s| s.to_string()),
                session_id: None, // UserContext doesn't have session_id
            }),
            http_context: HttpContextResponse {
                method: entry.http_context().method().to_string(),
                endpoint: entry.http_context().path().to_string(),
                status_code: entry.http_context().status_code(),
                response_time_ms: entry.http_context().response_time_ms(),
                user_agent: entry.http_context().user_agent().map(|s| s.to_string()),
                ip_address: entry.http_context().ip_address().map(|s| s.to_string()),
            },
            architecture_layer: entry.architecture_layer().to_string(),
            operation: entry.operation().to_string(),
            metadata: entry.metadata().to_json(),
            tags: entry.tags().to_vec(),
            analysis_status: entry.analysis_status().to_string(),
            related_log_ids: entry
                .related_log_ids()
                .iter()
                .map(|id| id.to_string())
                .collect(),
            created_at: entry.timestamp(),
            updated_at: Some(entry.timestamp()), // LogEntry doesn't have updated_at, using timestamp
        }
    }

    fn get_applied_filters(request: &LogSearchRequest) -> Vec<String> {
        let mut filters = Vec::new();

        if request.level.is_some() {
            filters.push("level".to_string());
        }
        if request.architecture_layer.is_some() {
            filters.push("architecture_layer".to_string());
        }
        if request.request_id.is_some() {
            filters.push("request_id".to_string());
        }
        if request.user_id.is_some() {
            filters.push("user_id".to_string());
        }
        if request.start_time.is_some() || request.end_time.is_some() {
            filters.push("time_range".to_string());
        }
        if request.message_contains.is_some() {
            filters.push("message_search".to_string());
        }
        if request.tags.is_some() {
            filters.push("tags".to_string());
        }
        if request.analysis_status.is_some() {
            filters.push("analysis_status".to_string());
        }

        filters
    }

    // ルーター設定
    pub fn router() -> Router<Arc<LogController>> {
        Router::new()
            .route("/search", get(Self::search_logs))
            .route("/logs/:id", get(Self::get_log_by_id))
            .route("/trace/:request_id", get(Self::get_logs_by_request_id))
            .route("/metrics", get(Self::get_performance_metrics))
            .route("/cleanup", post(Self::cleanup_logs))
            .route("/tags", put(Self::add_tags))
            .route("/tags", post(Self::remove_tag))
            .route("/status", put(Self::update_analysis_status))
            .route("/dashboard", get(Self::log_dashboard))
            .route("/aggregation", get(Self::get_log_aggregation))
    }
}
