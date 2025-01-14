use radius_sdk::kvstore::Model;
use serde::{Deserialize, Serialize};

use super::MerkleTree;
use crate::client::liveness::seeder::SequencerRpcInfo;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct RollupMetadata {
    pub rollup_block_height: u64,
    pub transaction_order: u64,

    pub merkle_tree: MerkleTree,

    pub cluster_id: String,

    pub platform_block_height: u64,
    pub is_leader: bool,
    pub leader_sequencer_rpc_info: SequencerRpcInfo,
}

impl RollupMetadata {
    pub fn new_merkle_tree(&mut self) {
        self.transaction_order = 0;
        self.merkle_tree = MerkleTree::new();
    }

    pub fn add_transaction_hash(&mut self, transaction_hash: &str) -> (u64, Vec<[u8; 32]>) {
        self.transaction_order += 1;
        self.merkle_tree.add_data(transaction_hash)
    }
}
