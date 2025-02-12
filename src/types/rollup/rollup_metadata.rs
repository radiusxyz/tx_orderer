use radius_sdk::kvstore::Model;
use serde::{Deserialize, Serialize};

use crate::client::liveness_service_manager::seeder::SequencerRpcInfo;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct RollupMetadata {
    pub rollup_block_height: u64,
    pub transaction_order: u64,
    pub cluster_id: String,
    pub platform_block_height: u64,
    pub is_leader: bool,
    pub leader_sequencer_rpc_info: SequencerRpcInfo,
    pub max_gas_limit: u64,
    pub current_gas: u64,
}
