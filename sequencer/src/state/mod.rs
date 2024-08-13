use std::{collections::HashMap, sync::Arc};

use futures::future::Shared;
use radius_sequencer_sdk::{
    context::{Context, SharedContext},
    liveness::publisher::Publisher,
};
use tokio::sync::Mutex;

use crate::{
    cli::Config,
    client::SeederClient,
    error::Error,
    types::{
        BlockHeight, Cluster, ClusterId, PvdeParams, RollupId, RollupMetadata, SequencingInfo,
        SequencingInfoKey, SigningKey,
    },
};

// Todo: split this mod into (internal, cluster, external)
pub struct InternalAppState {
    config: Config,

    sequencing_info: SequencingInfo,
    seeder_client: SeederClient,
    cluster_ids: ClusterId,
    publisher: Mutex<Option<Publisher>>,
}

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,

    // Todo remove
    rollup_metadatas: SharedContext<HashMap<RollupId, RollupMetadata>>,

    rollup_cluster_ids: SharedContext<HashMap<RollupId, ClusterId>>,
    sequencing_infos: SharedContext<HashMap<SequencingInfoKey, SequencingInfo>>,
    clusters: SharedContext<HashMap<ClusterId, Cluster>>,

    seeder_client: SeederClient,

    pvde_params: SharedContext<Option<PvdeParams>>,
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
        rollup_metadatas: HashMap<RollupId, RollupMetadata>,
        rollup_cluster_ids: HashMap<RollupId, ClusterId>,
        sequencing_infos: HashMap<SequencingInfoKey, SequencingInfo>,
        seeder_client: SeederClient,
        pvde_params: SharedContext<Option<PvdeParams>>,
    ) -> Self {
        let inner = AppStateInner {
            config,
            rollup_metadatas: SharedContext::from(rollup_metadatas),
            rollup_cluster_ids: SharedContext::from(rollup_cluster_ids),
            sequencing_infos: SharedContext::from(sequencing_infos),
            seeder_client,
            clusters: HashMap::new().into(),
            pvde_params: pvde_params,
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn block_height(&self, rollup_id: &RollupId) -> Result<BlockHeight, Error> {
        self.rollup_metadatas()
            .as_ref()
            .get(rollup_id)
            .map(|rollup_metadata| rollup_metadata.block_height())
            .ok_or(Error::NotFoundRollupMetadata)
    }

    pub fn rollup_metadatas(&self) -> Context<HashMap<RollupId, RollupMetadata>> {
        self.inner.rollup_metadatas.load()
    }

    pub fn rollup_metadata(&self, rollup_id: &RollupId) -> Result<RollupMetadata, Error> {
        self.inner
            .rollup_metadatas
            .load()
            .as_ref()
            .get(rollup_id)
            .cloned()
            .ok_or(Error::NotFoundRollupMetadata)
    }

    pub fn rollup_cluster_ids(&self) -> Context<HashMap<RollupId, ClusterId>> {
        self.inner.rollup_cluster_ids.load()
    }

    pub fn sequencing_infos(&self) -> Context<HashMap<SequencingInfoKey, SequencingInfo>> {
        self.inner.sequencing_infos.load()
    }

    pub fn sequencing_info(&self, key: &SequencingInfoKey) -> Result<SequencingInfo, Error> {
        self.inner
            .sequencing_infos
            .load()
            .as_ref()
            .get(key)
            .cloned()
            .ok_or(Error::NotFoundSequencingInfo)
    }

    pub fn clusters(&self) -> Context<HashMap<ClusterId, Cluster>> {
        self.inner.clusters.load()
    }

    pub fn signing_key(&self) -> &SigningKey {
        self.inner.config.signing_key()
    }

    pub fn seeder_client(&self) -> SeederClient {
        self.inner.seeder_client.clone()
    }

    pub fn cluster_id(&self, rollup_id: &RollupId) -> Result<ClusterId, Error> {
        self.inner
            .rollup_cluster_ids
            .load()
            .as_ref()
            .get(rollup_id)
            .cloned()
            .ok_or(Error::NotFoundClusterId)
    }

    pub fn cluster(&self, cluster_id: &ClusterId) -> Result<Cluster, Error> {
        self.inner
            .clusters
            .load()
            .as_ref()
            .get(cluster_id)
            .cloned()
            .ok_or(Error::NotFoundCluster)
    }

    pub fn pvde_params(&self) -> Context<Option<PvdeParams>> {
        self.inner.pvde_params.load()
    }

    pub fn set_rollup_metadata(&self, rollup_id: RollupId, rollup_metadata: RollupMetadata) {
        let mut rollup_metadatas = self.rollup_metadatas().as_ref().clone();

        rollup_metadatas.insert(rollup_id, rollup_metadata);

        self.inner.rollup_metadatas.store(rollup_metadatas);
    }

    pub fn set_cluster(&self, cluster: Cluster) {
        let mut new_clusters = self.inner.clusters.load().as_ref().clone();

        new_clusters.insert(cluster.cluster_id().clone(), cluster);

        self.inner.clusters.store(new_clusters);
    }

    pub fn set_cluster_id(&self, rollup_id: RollupId, cluster_id: ClusterId) {
        let mut rollup_cluster_ids = self.inner.rollup_cluster_ids.load().as_ref().clone();

        rollup_cluster_ids.insert(rollup_id, cluster_id);

        self.inner.rollup_cluster_ids.store(rollup_cluster_ids);
    }

    pub fn set_sequencing_info(&self, sequencing_info: SequencingInfo) {
        let mut new_sequencing_infos = self.inner.sequencing_infos.load().as_ref().clone();

        let sequencing_info_key = SequencingInfoKey::new(
            sequencing_info.platform.clone(),
            sequencing_info.sequencing_function_type.clone(),
            sequencing_info.service_type.clone(),
        );

        new_sequencing_infos.insert(sequencing_info_key, sequencing_info);

        self.inner.sequencing_infos.store(new_sequencing_infos);
    }
}
