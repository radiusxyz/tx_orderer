pub mod transaction_service;
pub mod batch_service;
pub mod validation_service;

pub use transaction_service::*;
pub use batch_service::*;
pub use validation_service::*;

use crate::state::AppState;

/// Service management for spawning background workers
pub struct ServiceManager;

impl ServiceManager {
    /// Initialize and spawn all background services
    pub async fn start_all_services(app_state: AppState) {
        Self::start_transaction_service(app_state.clone()).await;
        Self::start_batch_service(app_state.clone()).await;
        Self::start_validation_service(app_state).await;
    }

    async fn start_transaction_service(app_state: AppState) {
        tokio::spawn(async move {
            let service = TransactionService::new(app_state);
            service.run().await;
        });
    }

    async fn start_batch_service(app_state: AppState) {
        tokio::spawn(async move {
            let service = BatchService::new(app_state);
            service.run().await;
        });
    }

    async fn start_validation_service(app_state: AppState) {
        tokio::spawn(async move {
            let service = ValidationService::new(app_state);
            service.run().await;
        });
    }
}