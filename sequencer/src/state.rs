use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::{
    cli::Config,
    client::SeederClient,
    error::Error,
    types::{Cluster, ClusterId, RollupId, SequencingInfo, SequencingInfoKey, SigningKey},
};

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,

    rollup_cluster_ids: Mutex<HashMap<RollupId, ClusterId>>,
    sequencing_infos: Mutex<HashMap<SequencingInfoKey, SequencingInfo>>,

    seeder_client: SeederClient,
    clusters: Mutex<HashMap<ClusterId, Cluster>>,
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
        rollup_cluster_ids: HashMap<RollupId, ClusterId>,
        sequencing_infos: HashMap<SequencingInfoKey, SequencingInfo>,
        seeder_client: SeederClient,
    ) -> Self {
        let inner = AppStateInner {
            config,
            rollup_cluster_ids: Mutex::new(rollup_cluster_ids),
            sequencing_infos: Mutex::new(sequencing_infos),
            seeder_client,
            clusters: HashMap::new().into(),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub async fn sequencing_infos(&self) -> HashMap<SequencingInfoKey, SequencingInfo> {
        let sequencing_infos_lock = self.inner.sequencing_infos.lock().await;

        sequencing_infos_lock.clone()
    }

    pub fn signing_key(&self) -> &SigningKey {
        self.inner.config.signing_key()
    }

    pub fn seeder_client(&self) -> SeederClient {
        self.inner.seeder_client.clone()
    }

    pub async fn get_cluster_id(&self, rollup_id: &RollupId) -> Result<ClusterId, Error> {
        let rollup_cluster_ids_lock = self.inner.rollup_cluster_ids.lock().await;

        rollup_cluster_ids_lock
            .get(rollup_id)
            .cloned()
            .ok_or(Error::Uninitialized)
    }

    pub async fn set_cluster_id(&self, rollup_id: RollupId, cluster_id: ClusterId) {
        let mut rollup_cluster_ids_lock = self.inner.rollup_cluster_ids.lock().await;

        rollup_cluster_ids_lock.insert(rollup_id, cluster_id);
    }

    pub async fn get_cluster(&self, cluster_id: &ClusterId) -> Result<Cluster, Error> {
        let clusters_lock = self.inner.clusters.lock().await;

        clusters_lock
            .get(cluster_id)
            .cloned()
            .ok_or(Error::Uninitialized)
    }

    pub async fn set_cluster(&self, cluster: Cluster) {
        let mut clusters_lock = self.inner.clusters.lock().await;

        clusters_lock.insert(cluster.cluster_id().clone(), cluster);
    }

    pub async fn set_sequencing_info(&self, sequencing_info: SequencingInfo) {
        let mut sequencing_infos_lock = self.inner.sequencing_infos.lock().await;

        let sequencing_info_key = SequencingInfoKey::new(
            sequencing_info.platform.clone(),
            sequencing_info.sequencing_function_type.clone(),
            sequencing_info.service_type.clone(),
        );

        sequencing_infos_lock.insert(sequencing_info_key, sequencing_info);
    }
}
