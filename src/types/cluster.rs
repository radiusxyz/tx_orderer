use std::collections::btree_set::{self, BTreeSet};

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
    sequencer_rpc_url_list: Vec<SequencerRpcInfo>,
    rollup_id_list: Vec<String>,
    my_index: usize,
    block_margin: u64,
}

impl Cluster {
    pub fn new(
        sequencer_rpc_url_list: Vec<SequencerRpcInfo>,
        rollup_id_list: Vec<String>,
        my_index: usize,
        block_margin: u64,
    ) -> Self {
        Self {
            sequencer_rpc_url_list,
            rollup_id_list,
            my_index,
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
        let block_height_for_remove = platform_block_height.wrapping_sub(cluster.block_margin);

        let cluster_block_height = ClusterBlockHeight::new(block_height_for_remove + 1);

        ClusterBlockHeight::put(
            &cluster_block_height,
            platform,
            service_provider,
            cluster_id,
        )?;

        Cluster::delete(
            platform,
            service_provider,
            cluster_id,
            block_height_for_remove,
        )?;

        Ok(())
    }

    pub fn my_index(&self) -> usize {
        self.my_index
    }

    pub fn sequencer_list(&self) -> &Vec<SequencerRpcInfo> {
        &self.sequencer_rpc_url_list
    }

    pub fn rollup_id_list(&self) -> &Vec<String> {
        &self.rollup_id_list
    }

    pub fn block_margin(&self) -> u64 {
        self.block_margin
    }

    pub fn get_others_cluster_rpc_url_list(&self) -> Vec<String> {
        self.sequencer_rpc_url_list
            .iter()
            .enumerate()
            .filter_map(|(index, sequencer_rpc_info)| {
                if index != self.my_index {
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
        self.sequencer_rpc_url_list
            .iter()
            .enumerate()
            .filter_map(|(index, sequencer_rpc_info)| {
                if index != self.my_index {
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

    pub fn get_follower_cluster_rpc_url_list(
        &self,
        leader_address: &Address,
    ) -> Result<Vec<String>, Error> {
        let leader_index = self.get_leader_index(leader_address)?;

        Ok(self
            .sequencer_rpc_url_list
            .iter()
            .enumerate()
            .filter_map(|(index, sequencer_rpc_info)| {
                if index == leader_index {
                    None
                } else {
                    if sequencer_rpc_info.cluster_rpc_url.is_none() {
                        return None;
                    }
                    Some(sequencer_rpc_info.cluster_rpc_url.to_owned().unwrap())
                }
            })
            .collect())
    }

    pub fn get_leader_cluster_rpc_url(&self, leader_address: &Address) -> Result<String, Error> {
        let leader_index = self.get_leader_index(leader_address)?;

        self.sequencer_rpc_url_list
            .get(leader_index)
            .and_then(|sequencer_rpc_info| sequencer_rpc_info.cluster_rpc_url.clone())
            .ok_or(Error::EmptyLeaderClusterRpcUrl)
    }

    pub fn get_leader_external_rpc_url(&self, leader_address: &Address) -> Result<String, Error> {
        let leader_index = self.get_leader_index(leader_address)?;

        self.sequencer_rpc_url_list
            .get(leader_index)
            .and_then(|sequencer_rpc_info| sequencer_rpc_info.cluster_rpc_url.clone())
            .ok_or(Error::EmptyLeaderClusterRpcUrl)
    }

    pub fn get_leader_index(&self, leader_address: &Address) -> Result<usize, Error> {
        self.sequencer_rpc_url_list
            .iter()
            .position(|sequencer_rpc_info| sequencer_rpc_info.address == leader_address)
            .ok_or(Error::EmptyLeader)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, service_provider: ServiceProvider, cluster_id: &str))]
pub struct ClusterBlockHeight(u64);

impl ClusterBlockHeight {
    pub fn new(block_height: u64) -> Self {
        Self(block_height)
    }

    pub fn inner(&self) -> u64 {
        self.0
    }
}
