use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::{
    cli::Config,
    error::Error,
    types::{RollupCluster, RollupId},
};

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,
    rollup_clusters: Mutex<HashMap<RollupId, RollupCluster>>,
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
    pub fn new(config: Config) -> Self {
        let inner = AppStateInner {
            config,
            rollup_clusters: HashMap::new().into(),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub async fn get_rollup_cluster(&self, rollup_id: &RollupId) -> Result<RollupCluster, Error> {
        let rollup_clusters_lock = self.inner.rollup_clusters.lock().await;

        rollup_clusters_lock
            .get(rollup_id)
            .cloned()
            .ok_or(Error::Uninitialized)
    }

    pub async fn set_rollup_cluster(
        &mut self,
        rollup_id: &RollupId,
        rollup_cluster: RollupCluster,
    ) {
        let mut rollup_clusters_lock = self.inner.rollup_clusters.lock().await;

        rollup_clusters_lock.insert(rollup_id.clone(), rollup_cluster);
    }
}
