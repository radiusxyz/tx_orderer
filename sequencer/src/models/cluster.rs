use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadataModel {
    rollup_id: RollupId,
    pub rollup_block_height: BlockHeight,
    pub liveness_block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
    pub is_leader: bool,
}

impl ClusterMetadataModel {
    pub const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get(rollup_id: &RollupId) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_id);
        database()?.get(&key)
    }

    pub fn get_mut(rollup_id: &RollupId) -> Result<Lock<'static, Self>, database::Error> {
        let key = (Self::ID, rollup_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        let key = (Self::ID, self.rollup_id.clone());
        database()?.put(&key, self)
    }

    pub fn new(
        rollup_id: RollupId,
        liveness_block_height: BlockHeight,
        rollup_block_height: BlockHeight,
        transaction_order: TransactionOrder,
        is_leader: bool,
    ) -> Self {
        Self {
            rollup_id,
            liveness_block_height,
            rollup_block_height,
            transaction_order,
            is_leader,
        }
    }

    pub fn get_transaction_order(&self) -> TransactionOrder {
        self.transaction_order.clone()
    }

    pub fn increment_transaction_order(&mut self) {
        self.transaction_order.increment();
    }
}
