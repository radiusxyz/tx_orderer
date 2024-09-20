use super::prelude::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RollupMetadata {
    rollup_block_height: u64,
    transaction_order: u64,
    order_hash: OrderHash,

    is_leader: bool,
    platform_block_height: u64,

    cluster_id: String,
}

impl RollupMetadata {
    pub fn rollup_block_height(&self) -> u64 {
        self.rollup_block_height
    }

    pub fn transaction_order(&self) -> u64 {
        self.transaction_order
    }

    pub fn order_hash(&self) -> OrderHash {
        self.order_hash.clone()
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader
    }

    pub fn cluster_id(&self) -> &String {
        &self.cluster_id
    }

    pub fn platform_block_height(&self) -> u64 {
        self.platform_block_height
    }
}

impl RollupMetadata {
    pub fn set_is_leader(&mut self, is_leader: bool) {
        self.is_leader = is_leader;
    }

    pub fn set_cluster_id(&mut self, cluster_id: &String) {
        self.cluster_id = cluster_id.to_owned();
    }

    pub fn set_rollup_block_height(&mut self, block_height: u64) {
        self.rollup_block_height = block_height;
    }

    pub fn set_order_hash(&mut self, order_hash: OrderHash) {
        self.order_hash = order_hash;
    }

    pub fn set_transaction_order(&mut self, transaction_order: u64) {
        self.transaction_order = transaction_order;
    }

    pub fn set_platform_block_height(&mut self, platform_block_height: u64) {
        self.platform_block_height = platform_block_height;
    }

    pub fn increase_transaction_order(&mut self) -> u64 {
        self.transaction_order += 1;

        self.transaction_order
    }

    pub fn update_order_hash(&mut self, raw_transaction_hash: &RawTransactionHash) -> &OrderHash {
        self.order_hash.update_order_hash(raw_transaction_hash);
        &self.order_hash
    }
}
