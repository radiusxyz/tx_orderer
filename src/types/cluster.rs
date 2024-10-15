use std::collections::btree_set::{self, BTreeSet};

use super::prelude::*;
use crate::error::Error;

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
    sequencer_rpc_url_list: Vec<(String, Option<String>)>,
    rollup_id_list: Vec<String>,
    my_index: usize,
    block_margin: u64,
}

impl Cluster {
    pub fn new(
        sequencer_rpc_url_list: Vec<(String, Option<String>)>,
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

    pub fn sequencer_list(&self) -> &Vec<(String, Option<String>)> {
        &self.sequencer_rpc_url_list
    }

    pub fn rollup_id_list(&self) -> &Vec<String> {
        &self.rollup_id_list
    }

    pub fn block_margin(&self) -> u64 {
        self.block_margin
    }

    pub fn is_leader(&self, rollup_block_height: u64) -> bool {
        let leader_index = rollup_block_height as usize % self.sequencer_rpc_url_list.len();

        leader_index == self.my_index
    }

    pub fn get_others_rpc_url_list(&self) -> Vec<Option<String>> {
        self.sequencer_rpc_url_list
            .iter()
            .enumerate()
            .filter_map(|(index, (_address, rpc_url))| {
                if index != self.my_index {
                    Some(rpc_url.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_follower_rpc_url_list(&self, rollup_block_height: u64) -> Vec<Option<String>> {
        let leader_index = rollup_block_height as usize % self.sequencer_rpc_url_list.len();

        self.sequencer_rpc_url_list
            .iter()
            .enumerate()
            .filter_map(|(index, (_address, rpc_url))| {
                if index == leader_index {
                    None
                } else {
                    Some(rpc_url.clone())
                }
            })
            .collect()
    }

    pub fn get_leader_rpc_url(&self, rollup_block_height: u64) -> Option<String> {
        let leader_index = rollup_block_height as usize % self.sequencer_rpc_url_list.len();

        self.sequencer_rpc_url_list
            .get(leader_index)
            .and_then(|(_address, rpc_url)| rpc_url.clone())
    }

    pub fn get_leader_address(&self, rollup_block_height: u64) -> Result<String, Error> {
        let leader_index = rollup_block_height as usize % self.sequencer_rpc_url_list.len();

        self.sequencer_rpc_url_list
            .get(leader_index)
            .map(|(address, _rpc_url)| address.clone())
            .ok_or(Error::EmptyLeaderRpcUrl)
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
