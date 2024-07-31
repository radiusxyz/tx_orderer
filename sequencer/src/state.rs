use std::{collections::HashMap, sync::Arc};

use crate::{
    cli::Config,
    types::{Cluster, RollupId},
};

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,
    clusters: HashMap<RollupId, Cluster>,
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
            clusters: HashMap::new(),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    // TODO: it can be use multiple threads
    pub fn get_rollup_cluster(&self, rollup_id: &RollupId) -> Option<Cluster> {
        self.inner.clusters.get(rollup_id).cloned()
    }
}
