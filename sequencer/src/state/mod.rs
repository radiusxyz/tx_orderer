use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::{
    context::{Context, SharedContext},
    liveness::{publisher::Publisher, types::Block},
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

    rollup_states: SharedContext<HashMap<RollupId, RollupState>>,

    sequencing_infos: SharedContext<HashMap<SequencingInfoKey, SequencingInfo>>,
    clusters: SharedContext<HashMap<ClusterId, Cluster>>,

    seeder_client: SeederClient,

    // todo(jaemin): change Option<PvdeParams> to ZkpParams(3 variants)
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
        rollup_states: HashMap<RollupId, RollupState>,
        sequencing_infos: HashMap<SequencingInfoKey, SequencingInfo>,
        seeder_client: SeederClient,
        pvde_params: Option<PvdeParams>,
    ) -> Self {
        let inner = AppStateInner {
            config,
            rollup_states: SharedContext::from(rollup_states),
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

    pub fn issue_new_block(&self, rollup_id: &RollupId) -> Result<BlockHeight, Error> {
        let mut rollup_states = self.rollup_states().as_ref().clone();

        let rollup_state = rollup_states
            .get_mut(rollup_id)
            .ok_or(Error::NotFoundRollupState)?;

        rollup_state.rollup_block_height += 1;
        let rollup_block_height = rollup_state.rollup_block_height;

        self.inner.rollup_states.store(rollup_states);

        Ok(rollup_block_height)
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
            .rollup_states
            .load()
            .as_ref()
            .get(rollup_id)
            .and_then(|rollup_state| Some(rollup_state.cluster_id().clone()))
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

// Todo: Add fields
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupState {
    pub cluster_id: ClusterId,
    pub rollup_block_height: BlockHeight,
}

impl RollupState {
    pub fn new(cluster_id: ClusterId, block_height: BlockHeight) -> Self {
        Self {
            cluster_id,
            rollup_block_height: block_height,
        }
    }

    pub fn cluster_id(&self) -> &ClusterId {
        &self.cluster_id
    }

    pub fn block_height(&self) -> BlockHeight {
        self.rollup_block_height
    }
}
