use std::sync::Arc;

use crate::{client::liveness::seeder::SeederClient, types::*};

pub struct AppState {
    inner: Arc<AppStateInner>,
}
struct AppStateInner {
    config: Config,
    seeder_client: SeederClient,
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
    pub fn new(config: Config, seeder_client: SeederClient) -> Self {
        let inner = AppStateInner {
            config,
            seeder_client,
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
}
