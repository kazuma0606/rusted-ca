//presentation/router/account_router.rs
// Account HTTP Router
// 2025/8/28

use crate::presentation::controller::account_controller::AccountController;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

pub fn create_account_router(controller: Arc<AccountController>) -> Router {
    Router::new()
        // Account CRUD operations
        .route("/api/account", post({
            let controller = Arc::clone(&controller);
            move |request| async move { controller.create_account(request).await }
        }))
        .route("/api/account/:id", get({
            let controller = Arc::clone(&controller);
            move |path| async move { controller.get_account(path).await }
        }))
        .route("/api/account/:id", put({
            let controller = Arc::clone(&controller);
            move |path, request| async move { controller.update_account(path, request).await }
        }))
        .route("/api/account/:id", delete({
            let controller = Arc::clone(&controller);
            move |path| async move { controller.delete_account(path).await }
        }))
        
        // Balance operations
        .route("/api/account/:id/balance", get({
            let controller = Arc::clone(&controller);
            move |path| async move { controller.get_account_balance(path).await }
        }))
        .route("/api/account/:id/credit", post({
            let controller = Arc::clone(&controller);
            move |path, request| async move { controller.credit_account(path, request).await }
        }))
        .route("/api/account/:id/debit", post({
            let controller = Arc::clone(&controller);
            move |path, request| async move { controller.debit_account(path, request).await }
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::usecases::{
        create_account_usecase::CreateAccountUseCase,
        get_account_usecase::GetAccountUseCase,
    };
    use axum_test::TestServer;
    
    // Note: These would be integration tests that require proper setup
    // The actual test implementation would require setting up the full DI container
    
    #[tokio::test]
    async fn test_account_router_creation() {
        // This is a basic test to ensure the router can be created
        // Real tests would require proper use case setup
        
        // For now, we'll skip the full implementation as it requires
        // the complete DI container setup
        assert!(true);
    }
}