mod model;

use std::collections::btree_set::{BTreeSet, Iter};

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

    pub fn iter(&self) -> Iter<'_, String> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterInfo {
    sequencer_address_list: Vec<String>,
    block_margin: u64,
}

impl ClusterInfo {
    pub fn new(sequencer_address_list: Vec<String>, block_margin: u64) -> Self {
        Self {
            sequencer_address_list,
            block_margin,
        }
    }

    pub fn block_margin(&self) -> u64 {
        self.block_margin
    }

    pub fn sequencer_address_list(&self) -> &Vec<String> {
        &self.sequencer_address_list
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadata {
    leader_index: usize,
    rollup_block_height: u64,
    transaction_order: u64,
    sequencer_address_list: Vec<String>,
}

// impl ClusterMetadata {
//     pub fn new(leader_index: usize, rollup_block_height: u64) -> Self {
//         Self {
//             leader_index,
//             rollup_block_height,
//             transaction_order: 0,
//             sequencer_address_list,
//         }
//     }
// }
