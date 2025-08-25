use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogSearchRequest {
    pub level: Option<String>,
    pub architecture_layer: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub message_contains: Option<String>,
    pub tags: Option<Vec<String>>,
    pub analysis_status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl Default for LogSearchRequest {
    fn default() -> Self {
        Self {
            level: None,
            architecture_layer: None,
            request_id: None,
            user_id: None,
            start_time: None,
            end_time: None,
            message_contains: None,
            tags: None,
            analysis_status: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogPerformanceMetricsRequest {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogCleanupRequest {
    pub cutoff_time: DateTime<Utc>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTagsRequest {
    pub log_id: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveTagRequest {
    pub log_id: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddRelatedLogRequest {
    pub log_id: String,
    pub related_log_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAnalysisStatusRequest {
    pub log_id: String,
    pub status: String,
}