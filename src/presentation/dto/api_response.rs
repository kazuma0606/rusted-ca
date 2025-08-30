//presentation/dto/api_response.rs
// 統一APIレスポンス
// 2025/7/8

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub request_id: String,
    pub processing_time_ms: u64,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Success".to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            processing_time_ms: 0, // TODO: Implement actual timing
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message,
            request_id: uuid::Uuid::new_v4().to_string(),
            processing_time_ms: 0,
        }
    }
}
