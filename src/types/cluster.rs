use std::collections::btree_set::{BTreeSet, Iter};

use radius_sequencer_sdk::liveness_radius::types::U256;

use super::prelude::*;

/// 09/05
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ClusterInfo {
    sequencer_info: Vec<(String, Option<String>)>,
    rollup_info: Vec<(String, Option<String>)>,
    block_margin: u64,
}

impl ClusterInfo {
    pub fn new(
        sequencer_info: Vec<(String, Option<String>)>,
        rollup_info: Vec<(String, Option<String>)>,
        block_margin: u64,
    ) -> Self {
        Self {
            sequencer_info,
            rollup_info,
            block_margin,
        }
    }

    pub fn block_margin(&self) -> u64 {
        self.block_margin
    }
}
