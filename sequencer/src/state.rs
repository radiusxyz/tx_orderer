use std::sync::Arc;

use ssal::avs::SsalClient;

use crate::config::Config;

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,
    ssal_client: SsalClient,
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
    pub fn new(config: Config, ssal_client: SsalClient) -> Self {
        let inner = AppStateInner {
            config,
            ssal_client,
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn ssal_client(&self) -> SsalClient {
        self.inner.ssal_client.clone()
    }
}
