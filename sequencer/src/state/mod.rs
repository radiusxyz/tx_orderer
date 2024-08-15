use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::{
    context::{Context, SharedContext},
    liveness::publisher::Publisher,
};
use serde::{Deserialize, Serialize};
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
    publisher: Mutex<Option<Publisher>>,
}

pub struct ExternalAppState {
    config: Config,

    rollup_metadatas: SharedContext<HashMap<RollupId, RollupMetadata>>,
}

pub struct ClusterAppState {}

pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: Config,

    // Todo: change add field or remove. now it has only block_height
    rollup_states: SharedContext<HashMap<RollupId, RollupState>>,

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

// TODO(jaemin): 필드값 아니면 getter로 사용
impl AppState {
    pub fn new(
        config: Config,
        rollup_states: HashMap<RollupId, RollupState>,
        rollup_cluster_ids: HashMap<RollupId, ClusterId>,
        sequencing_infos: HashMap<SequencingInfoKey, SequencingInfo>,
        seeder_client: SeederClient,
        pvde_params: Option<PvdeParams>,
    ) -> Self {
        let inner = AppStateInner {
            config,
            rollup_states: SharedContext::from(rollup_states),
            rollup_cluster_ids: SharedContext::from(rollup_cluster_ids),
            sequencing_infos: SharedContext::from(sequencing_infos),
            seeder_client,
            clusters: HashMap::new().into(),
            pvde_params: SharedContext::from(pvde_params),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub fn rollup_states(&self) -> Context<HashMap<RollupId, RollupState>> {
        self.inner.rollup_states.load()
    }

    pub fn rollup_cluster_ids(&self) -> Context<HashMap<RollupId, ClusterId>> {
        self.inner.rollup_cluster_ids.load()
    }

    pub fn sequencing_infos(&self) -> Context<HashMap<SequencingInfoKey, SequencingInfo>> {
        self.inner.sequencing_infos.load()
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

    pub fn pvde_params(&self) -> Context<Option<PvdeParams>> {
        self.inner.pvde_params.load()
    }

    // Todo: change type
    pub fn get_block_height(&self, rollup_id: &RollupId) -> Result<BlockHeight, Error> {
        self.rollup_states()
            .as_ref()
            .get(rollup_id)
            .map(|rollup_metadata| rollup_metadata.block_height())
            .ok_or(Error::NotFoundRollupState)
    }

    pub fn get_cluster_id(&self, rollup_id: &RollupId) -> Result<ClusterId, Error> {
        self.inner
            .rollup_cluster_ids
            .load()
            .as_ref()
            .get(rollup_id)
            .cloned()
            .ok_or(Error::NotFoundClusterId)
    }

    pub fn get_cluster(&self, cluster_id: &ClusterId) -> Result<Cluster, Error> {
        self.inner
            .clusters
            .load()
            .as_ref()
            .get(cluster_id)
            .cloned()
            .ok_or(Error::NotFoundCluster)
    }

    pub fn get_sequencing_info(&self, key: &SequencingInfoKey) -> Result<SequencingInfo, Error> {
        self.inner
            .sequencing_infos
            .load()
            .as_ref()
            .get(key)
            .cloned()
            .ok_or(Error::NotFoundSequencingInfo)
    }

    pub fn get_rollup_state(&self, rollup_id: &RollupId) -> Result<RollupState, Error> {
        self.inner
            .rollup_states
            .load()
            .as_ref()
            .get(rollup_id)
            .cloned()
            .ok_or(Error::NotFoundRollupState)
    }

    pub fn set_rollup_state(&self, rollup_id: RollupId, rollup_state: RollupState) {
        let mut rollup_states = self.rollup_states().as_ref().clone();

        rollup_states.insert(rollup_id, rollup_state);

        self.inner.rollup_states.store(rollup_states);
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

// Todo: Add fields or remove
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupState {
    pub block_height: BlockHeight,
}

impl RollupState {
    pub fn new(block_height: BlockHeight) -> Self {
        Self { block_height }
    }

    pub fn block_height(&self) -> BlockHeight {
        self.block_height
    }
}
