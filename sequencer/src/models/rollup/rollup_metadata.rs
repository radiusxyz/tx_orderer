use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupMetadataModel {
    cluster_id: ClusterId,
    rollup_id: RollupId,

    rollup_block_height: BlockHeight,
    liveness_block_height: BlockHeight,
    transaction_order: TransactionOrder,
    is_leader: bool,
}

impl RollupMetadataModel {
    pub fn new(cluster_id: ClusterId, rollup_id: RollupId) -> Self {
        Self {
            cluster_id,
            rollup_id,
            rollup_block_height: 0,
            liveness_block_height: 0,
            transaction_order: 0.into(),
            is_leader: false,
        }
    }

    pub fn get_transaction_order(&self) -> TransactionOrder {
        self.transaction_order.clone()
    }

    pub fn increment_transaction_order(&mut self) {
        self.transaction_order.increment();
    }

    pub fn rollup_block_height(&self) -> BlockHeight {
        self.rollup_block_height.clone()
    }

    pub fn liveness_block_height(&self) -> BlockHeight {
        self.liveness_block_height.clone()
    }

    pub fn transaction_order(&self) -> TransactionOrder {
        self.transaction_order.clone()
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
}

impl RollupMetadataModel {
    pub const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get(rollup_id: &RollupId) -> Result<Self, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get(&key)
    }

    pub fn get_mut(rollup_id: &RollupId) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, rollup_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (Self::ID, self.cluster_id.clone());
        database()?.put(&key, self)
    }
}
