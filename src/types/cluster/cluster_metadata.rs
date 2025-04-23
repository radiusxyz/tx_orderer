use radius_sdk::kvstore::Model;
use serde::{Deserialize, Serialize};

use crate::{
    client::seeder::TxOrdererRpcInfo,
    types::{LivenessServiceProvider, Platform},
};

use super::ClusterId;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, liveness_service_provider: LivenessServiceProvider, cluster_id: &str))]
pub struct ClusterMetadata {
    pub cluster_id: ClusterId,
    pub platform_block_height: u64,

    pub is_leader: bool,
    pub leader_tx_orderer_rpc_info: Option<TxOrdererRpcInfo>,
}

impl ClusterMetadata {
    pub fn new(cluster_id: ClusterId, platform_block_height: u64) -> Self {
        Self {
            cluster_id,
            platform_block_height,
            is_leader: false,
            leader_tx_orderer_rpc_info: None,
        }
    }
}
