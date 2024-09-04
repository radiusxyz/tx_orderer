use std::collections::btree_set::{BTreeSet, Iter};

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
    // The list of (Address, RPC URL) tuple.
    sequencer_rpc_url_list: Vec<(String, String)>,
    // Todo: Rollup Metadata
    rollup_metadata: Vec<String>,
}
