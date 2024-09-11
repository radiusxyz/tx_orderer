mod model;

use std::{
    collections::btree_set::{self, BTreeSet},
    slice,
};

pub use model::*;

use super::prelude::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterInfo {
    sequencer_rpc_url_list: Vec<(String, Option<String>)>,
    rollup_id_list: Vec<String>,
    my_index: usize,
    block_margin: u64,
}

impl ClusterInfo {
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadata {
    leader_index: usize,
    my_index: usize,
    sequencer_list: Vec<(String, Option<String>)>,
}

impl ClusterMetadata {
    pub fn new(
        leader_index: usize,
        my_index: usize,
        sequencer_list: Vec<(String, Option<String>)>,
    ) -> Self {
        Self {
            leader_index,
            my_index,
            sequencer_list,
        }
    }

    pub fn is_leader(&self) -> bool {
        self.leader_index == self.my_index
    }

    pub fn leader(&self) -> Option<String> {
        self.sequencer_list
            .get(self.leader_index)
            .unwrap()
            .1
            .clone()
    }

    /// Return the list of RPC URLs except the leader's.
    pub fn followers(&self) -> Vec<Option<String>> {
        self.sequencer_list
            .iter()
            .enumerate()
            .filter_map(|(index, (_address, rpc_url))| {
                if index == self.leader_index {
                    Some(rpc_url.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Return the list of RPC URL except mine.
    pub fn others(&self) -> Vec<Option<String>> {
        self.sequencer_list
            .iter()
            .enumerate()
            .filter_map(|(index, (_address, rpc_url))| {
                if index == self.my_index {
                    Some(rpc_url.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Return the iterator for the sequencer list.
    pub fn iter(&self) -> slice::Iter<'_, (String, Option<String>)> {
        self.sequencer_list.iter()
    }
}
