use std::sync::Arc;

use ssal::avs::SsalClient;

use crate::{config::Config, types::Cluster};

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,
    ssal_client: SsalClient,
    cluster: Cluster,
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
    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn ssal_client(&self) -> SsalClient {
        self.inner.ssal_client.clone()
    }

    pub fn cluster(&self) -> Cluster {
        self.inner.cluster.clone()
    }
}
