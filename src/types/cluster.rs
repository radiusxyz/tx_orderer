use std::collections::{
    btree_set::{self, BTreeSet},
    BTreeMap,
};

use super::prelude::*;
use crate::{
    client::liveness_service_manager::{
        radius::{initialize_new_cluster, LivenessClient},
        seeder::SequencerRpcInfo,
    },
    error::Error,
    state::AppState,
};

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, service_provider: ServiceProvider, cluster_id: &str, platform_block_height: u64))]
pub struct Cluster {
    #[serde(serialize_with = "serialize_address")]
    pub sequencer_address: Address,

    pub rollup_id_list: BTreeSet<String>,
    pub sequencer_rpc_infos: BTreeMap<usize, SequencerRpcInfo>,

    pub block_margin: u64,
}

impl Cluster {
    pub fn new(
        sequencer_rpc_infos: BTreeMap<usize, SequencerRpcInfo>,
        rollup_id_list: BTreeSet<String>,
        sequencer_address: Address,
        block_margin: u64,
    ) -> Self {
        Self {
            sequencer_rpc_infos,
            rollup_id_list,
            sequencer_address,
            block_margin,
        }
    }

    pub async fn put_and_update_with_margin(
        cluster: &Cluster,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &str,
        platform_block_height: u64,
    ) -> Result<(), KvStoreError> {
        Cluster::put(
            cluster,
            platform,
            service_provider,
            cluster_id,
            platform_block_height,
        )?;

        // Keep [`ClusterInfo`] for `Self::Margin` blocks.
        let block_height_for_remove = platform_block_height.wrapping_sub(cluster.block_margin * 2);

        Cluster::delete(
            platform,
            service_provider,
            cluster_id,
            block_height_for_remove,
        )?;

        Ok(())
    }

    pub fn get_sequencer_address_list(&self) -> Vec<Address> {
        self.sequencer_rpc_infos
            .values()
            .map(|sequencer_rpc_info| sequencer_rpc_info.address.clone())
            .collect()
    }

    pub fn get_others_cluster_rpc_url_list(&self) -> Vec<String> {
        self.sequencer_rpc_infos
            .values()
            .filter_map(|sequencer_rpc_info| {
                if sequencer_rpc_info.address != self.sequencer_address {
                    if sequencer_rpc_info.cluster_rpc_url.is_none() {
                        return None;
                    }

                    Some(sequencer_rpc_info.cluster_rpc_url.to_owned().unwrap())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_others_external_rpc_url_list(&self) -> Vec<String> {
        self.sequencer_rpc_infos
            .values()
            .filter_map(|sequencer_rpc_info| {
                if sequencer_rpc_info.address != self.sequencer_address {
                    if sequencer_rpc_info.external_rpc_url.is_none() {
                        return None;
                    }

                    Some(sequencer_rpc_info.external_rpc_url.to_owned().unwrap())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_sequencer_rpc_info(&self, address: &Address) -> Option<SequencerRpcInfo> {
        self.sequencer_rpc_infos
            .iter()
            .find(|(_index, sequencer_rpc_info)| sequencer_rpc_info.address == address)
            .map(|(_index, sequencer_rpc_info)| sequencer_rpc_info.clone())
    }

    pub fn register_sequencer(&mut self, index: usize, sequencer_rpc_info: SequencerRpcInfo) {
        self.sequencer_rpc_infos.insert(index, sequencer_rpc_info);
    }

    pub fn deregister_sequencer(&mut self, sequencer_address: &str) {
        let sequencer_index = self
            .sequencer_rpc_infos
            .iter()
            .find(|(_index, sequencer_rpc_info)| sequencer_rpc_info.address == sequencer_address)
            .map(|(index, _sequencer)| *index);

        if let Some(sequencer_index) = sequencer_index {
            self.sequencer_rpc_infos.remove(&sequencer_index);
        }
    }

    pub fn add_rollup(&mut self, rollup_id: &str) {
        self.rollup_id_list.insert(rollup_id.to_owned());
    }
}

impl Cluster {
    pub async fn sync_cluster(
        context: AppState,
        cluster_id: &str,
        liveness_client: &LivenessClient,
        platform_block_height: u64,
    ) -> Result<Cluster, Error> {
        let block_margin: u64 = liveness_client
            .publisher()
            .get_block_margin()
            .await
            .expect("Failed to get block margin")
            .try_into()
            .expect("Failed to convert block margin");

        initialize_new_cluster(
            context,
            liveness_client,
            cluster_id,
            platform_block_height,
            block_margin,
        )
        .await
        .unwrap();

        Cluster::get(
            liveness_client.platform(),
            liveness_client.service_provider(),
            cluster_id,
            platform_block_height,
        ).map_err(|e| {
            tracing::error!(
                "Failed to retrieve cluster - cluster_id: {:?} / platform_block_height: {:?} / error: {:?}",
                cluster_id,
                platform_block_height,
                e
            );

            Error::ClusterNotFound
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, service_provider: ServiceProvider))]
pub struct ClusterIdList(BTreeSet<String>);

impl ClusterIdList {
    pub fn insert(&mut self, cluster_id: impl AsRef<str>) {
        self.0.insert(cluster_id.as_ref().into());
    }

    pub fn remove(&mut self, cluster_id: impl AsRef<str>) {
        self.0.remove(cluster_id.as_ref());
    }

    pub fn iter(&self) -> btree_set::Iter<'_, String> {
        self.0.iter()
    }
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, service_provider: ServiceProvider, cluster_id: &str))]
pub struct LatestClusterBlockHeight(u64);

impl LatestClusterBlockHeight {
    pub fn get_block_height(&self) -> u64 {
        self.0
    }

    pub fn set_block_height(&mut self, block_height: u64) {
        self.0 = block_height;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(cluster_id: &str, platform_block_height: u64))]
pub struct LivenessEventList(Vec<LivenessEventType>);

impl LivenessEventList {
    pub fn push(&mut self, event_type: LivenessEventType) {
        self.0.push(event_type.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = &LivenessEventType> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LivenessEventType {
    RegisteredSequencer(usize, SequencerRpcInfo),
    DeregisteredSequencer(String),
    AddedRollup(String, String),
}
