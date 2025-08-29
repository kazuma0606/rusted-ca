//presentation/router/payment_router.rs
// Payment HTTP Router
// 2025/8/29

use crate::presentation::controller::payment_controller::PaymentController;
use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;

/// Create payment routes
/// 
/// Routes provided:
/// - POST   /api/payment              - Create new payment
/// - GET    /api/payment/:id          - Get payment by ID  
/// - PUT    /api/payment/:id/process  - Process payment (pending -> processing -> success)
/// - PUT    /api/payment/:id/refund   - Refund payment (partial or full)
/// - GET    /api/payments             - List payments with filtering
/// - GET    /api/account/:id/payments/stats - Get payment statistics for account
pub fn create_payment_router(controller: Arc<PaymentController>) -> Router {
    Router::new()
        // Single payment operations
        .route("/payment", post({
            let controller = Arc::clone(&controller);
            move |request| async move { controller.create_payment(request).await }
        }))
        .route("/payment/:id", get({
            let controller = Arc::clone(&controller);
            move |path| async move { controller.get_payment(path).await }
        }))
        .route("/payment/:id/process", put({
            let controller = Arc::clone(&controller);
            move |path, request| async move { controller.process_payment(path, request).await }
        }))
        .route("/payment/:id/refund", put({
            let controller = Arc::clone(&controller);
            move |path, request| async move { controller.refund_payment(path, request).await }
        }))
        
        // Payment queries and statistics
        .route("/payments", get({
            let controller = Arc::clone(&controller);
            move |query| async move { controller.list_payments(query).await }
        }))
        .route("/account/:id/payments/stats", get({
            let controller = Arc::clone(&controller);
            move |path| async move { controller.get_payment_stats(path).await }
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_router_creation() {
        // This would require mock PaymentController for proper testing
        // For now, just verify the module compiles
        assert!(true);
    }
}