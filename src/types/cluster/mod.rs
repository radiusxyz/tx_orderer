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
    platform: Platform,
    service_provider: ServiceProvider,
    sequencer_info: Vec<String>,
    rollup_info: Vec<(String,)>,
    block_margin: u64,
}

impl ClusterInfo {
    pub fn block_margin(&self) -> u64 {
        self.block_margin
    }
}
