use std::sync::Arc;

use ssal::avs::SsalClient;
use tokio::sync::Mutex;

use crate::{config::Config, error::Error, types::Cluster};

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,
    ssal_client: SsalClient,
    cluster: Mutex<Option<Cluster>>,
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
    pub fn new(config: Config, ssal_client: SsalClient, cluster: Option<Cluster>) -> Self {
        let inner = AppStateInner {
            config,
            ssal_client,
            cluster: Mutex::new(cluster),
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

    pub async fn cluster(&self) -> Result<Cluster, Error> {
        let cluster_lock = self.inner.cluster.lock().await;
        match &*cluster_lock {
            Some(cluster) => Ok(cluster.clone()),
            None => Err(Error::Uninitialized),
        }
    }

    pub async fn update_cluster(&self, cluster: Cluster) {
        let mut cluster_lock = self.inner.cluster.lock().await;
        *cluster_lock = Some(cluster);
    }
}
