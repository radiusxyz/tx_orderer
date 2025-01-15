use std::collections::{
    btree_set::{self, BTreeSet},
    BTreeMap,
};

use super::prelude::*;
use crate::{client::liveness::seeder::SequencerRpcInfo, error::Error};

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

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, service_provider: ServiceProvider, cluster_id: &str, platform_block_height: u64))]
pub struct Cluster {
    pub sequencer_rpc_infos: BTreeMap<usize, SequencerRpcInfo>,
    pub rollup_id_list: Vec<String>,

    #[serde(serialize_with = "serialize_address")]
    pub address: Address,
    pub block_margin: u64,
}

impl Cluster {
    pub fn new(
        sequencer_rpc_infos: BTreeMap<usize, SequencerRpcInfo>,
        rollup_id_list: Vec<String>,
        address: Address,
        block_margin: u64,
    ) -> Self {
        Self {
            sequencer_rpc_infos,
            rollup_id_list,
            address,
            block_margin,
        }
    }

    pub fn put_and_update_with_margin(
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

    pub fn get_others_cluster_rpc_url_list(&self) -> Vec<String> {
        self.sequencer_rpc_infos
            .values()
            .filter_map(|sequencer_rpc_info| {
                if sequencer_rpc_info.address != self.address {
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
                if sequencer_rpc_info.address != self.address {
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

    pub fn get_follower_cluster_rpc_url_list(&self, rollup_block_height: u64) -> Vec<String> {
        let leader_index = self.get_leader_index(rollup_block_height);

        self.sequencer_rpc_infos
            .iter()
            .filter_map(|(index, sequencer_rpc_info)| {
                if *index == leader_index {
                    None
                } else {
                    if sequencer_rpc_info.cluster_rpc_url.is_none() {
                        return None;
                    }
                    Some(sequencer_rpc_info.cluster_rpc_url.to_owned().unwrap())
                }
            })
            .collect()
    }

    pub fn get_leader_cluster_rpc_url(&self, rollup_block_height: u64) -> Result<String, Error> {
        let leader_index = self.get_leader_index(rollup_block_height);

        self.sequencer_rpc_infos
            .get(&leader_index)
            .and_then(|sequencer_rpc_info| sequencer_rpc_info.cluster_rpc_url.clone())
            .ok_or(Error::EmptyLeaderClusterRpcUrl)
    }

    pub fn get_leader_external_rpc_url(&self, rollup_block_height: u64) -> Result<String, Error> {
        let leader_index = self.get_leader_index(rollup_block_height);

        self.sequencer_rpc_infos
            .get(&leader_index)
            .and_then(|sequencer_rpc_info| sequencer_rpc_info.external_rpc_url.clone())
            .ok_or(Error::EmptyLeaderClusterRpcUrl)
    }

    pub fn get_leader_address(&self, rollup_block_height: u64) -> Result<Address, Error> {
        let leader_index = self.get_leader_index(rollup_block_height);

        self.sequencer_rpc_infos
            .get(&leader_index)
            .map(|sequencer_rpc_info| sequencer_rpc_info.address.clone())
            .ok_or(Error::EmptyLeader)
    }

    pub fn get_sequencer_rpc_info(&self, address: &Address) -> Option<SequencerRpcInfo> {
        self.sequencer_rpc_infos
            .iter()
            .find(|(_index, sequencer_rpc_info)| sequencer_rpc_info.address == address)
            .map(|(_index, sequencer_rpc_info)| sequencer_rpc_info.clone())
    }

    pub fn get_leader_index(&self, rollup_block_height: u64) -> usize {
        rollup_block_height as usize % self.sequencer_rpc_infos.len()
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
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(cluster_id: &String, platform_block_height: u64))]
pub struct LivenessEventList(Vec<LivenessEventType>);

impl LivenessEventList {
    pub fn push(&mut self, event_type: impl Into<LivenessEventType>) {
        self.0.push(event_type.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = &LivenessEventType> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LivenessEventType {
    RegisteredSequencer((usize, SequencerRpcInfo)),
    DeregisteredSequencer(String),
}

impl From<(usize, SequencerRpcInfo)> for LivenessEventType {
    fn from(value: (usize, SequencerRpcInfo)) -> Self {
        Self::RegisteredSequencer(value)
    }
}

impl From<String> for LivenessEventType {
    fn from(value: String) -> Self {
        Self::DeregisteredSequencer(value)
    }
}
