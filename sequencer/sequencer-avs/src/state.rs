use std::sync::Arc;

use database::Database;
use ssal::avs::SsalClient;

use crate::config::Config;

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,
    database: Database,
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
    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn database(&self) -> Database {
        self.inner.database.clone()
    }

    pub fn ssal_client(&self) -> SsalClient {
        self.inner.ssal_client.clone()
    }
}
