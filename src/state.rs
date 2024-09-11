use std::sync::Arc;

use crate::{
    client::liveness::{key_management_system::KeyManagementSystemClient, seeder::SeederClient},
    types::*,
};

pub struct AppState {
    inner: Arc<AppStateInner>,
}
struct AppStateInner {
    config: Config,
    seeder_client: SeederClient,
    key_management_client: KeyManagementSystemClient,
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl AppState {
    pub fn new(
        config: Config,
        seeder_client: SeederClient,
        key_management_system_client: KeyManagementSystemClient,
    ) -> Self {
        let inner = AppStateInner {
            config,
            seeder_client,
            key_management_client: key_management_system_client,
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn seeder_client(&self) -> &SeederClient {
        &self.inner.seeder_client
    }

    pub fn key_management_system_client(&self) -> &KeyManagementSystemClient {
        &self.inner.key_management_client
    }
}
