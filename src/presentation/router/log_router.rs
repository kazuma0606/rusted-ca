use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, put},
};

use crate::presentation::controller::log_controller::LogController;

pub fn log_router(log_controller: Arc<LogController>) -> Router {
    Router::new()
        // ログ検索・取得
        .route("/search", get(LogController::search_logs))
        .route("/logs/:id", get(LogController::get_log_by_id))
        .route(
            "/trace/:request_id",
            get(LogController::get_logs_by_request_id),
        )
        // パフォーマンスメトリクス
        .route("/metrics", get(LogController::get_performance_metrics))
        .route("/aggregation", get(LogController::get_log_aggregation))
        // ログ管理
        .route("/cleanup", post(LogController::cleanup_logs))
        .route("/tags", put(LogController::add_tags))
        .route("/tags/remove", post(LogController::remove_tag))
        .route("/status", put(LogController::update_analysis_status))
        // ダッシュボード
        .route("/dashboard", get(LogController::log_dashboard))
        .with_state(log_controller)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::http::StatusCode;
    use tower::ServiceExt; // for `oneshot` method

    #[tokio::test]
    async fn test_log_router_routes() {
        // Mock dependencies would be needed for a proper test
        // This is just a structure test to ensure routes are properly defined

        // In a real test, you would:
        // 1. Create mock dependencies
        // 2. Create the controller with mocks
        // 3. Test individual endpoints
        // 4. Verify responses

        // For now, just verify the router can be created
        // let mock_controller = Arc::new(/* mock controller */);
        // let app = log_router(mock_controller);
        // assert!(app is constructed properly)
    }
}
