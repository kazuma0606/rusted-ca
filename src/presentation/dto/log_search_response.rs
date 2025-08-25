use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogSearchResponse {
    pub logs: Vec<LogEntryResponse>,
    pub total_count: usize,
    pub has_more: bool,
    pub search_metadata: SearchMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntryResponse {
    pub log_id: String,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub request_id: String,
    pub user_context: Option<UserContextResponse>,
    pub http_context: HttpContextResponse,
    pub architecture_layer: String,
    pub operation: String,
    pub metadata: serde_json::Value,
    pub tags: Vec<String>,
    pub analysis_status: String,
    pub related_log_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserContextResponse {
    pub user_id: String,
    pub username: Option<String>,
    pub role: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpContextResponse {
    pub method: String,
    pub endpoint: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub query_time_ms: u64,
    pub search_criteria: String,
    pub filters_applied: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogPerformanceMetricsResponse {
    pub total_requests: usize,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: u64,
    pub error_rate: f64,
    pub requests_by_status_code: HashMap<u16, usize>,
    pub requests_by_layer: HashMap<String, usize>,
    pub time_range: TimeRangeResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeRangeResponse {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogCleanupResponse {
    pub deleted_count: usize,
    pub dry_run: bool,
    pub execution_time_ms: u64,
    pub cutoff_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogAggregationResponse {
    pub error_summary: ErrorSummary,
    pub performance_overview: PerformanceOverview,
    pub activity_timeline: Vec<ActivityTimelineEntry>,
    pub top_errors: Vec<TopErrorEntry>,
    pub slowest_endpoints: Vec<SlowEndpointEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub total_errors: usize,
    pub critical_errors: usize,
    pub error_rate_24h: f64,
    pub most_common_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceOverview {
    pub avg_response_time_24h: f64,
    pub p95_response_time_24h: u64,
    pub throughput_requests_per_minute: f64,
    pub active_requests: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityTimelineEntry {
    pub timestamp: DateTime<Utc>,
    pub request_count: usize,
    pub error_count: usize,
    pub avg_response_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopErrorEntry {
    pub message: String,
    pub count: usize,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub affected_endpoints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlowEndpointEntry {
    pub endpoint: String,
    pub avg_response_time_ms: f64,
    pub request_count: usize,
    pub slowest_request_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}