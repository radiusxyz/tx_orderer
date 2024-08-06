use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupMetadataModel {
    cluster_id: ClusterId,
    rollup_id: RollupId,

    pub rollup_block_height: BlockHeight,
    pub liveness_block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
    pub is_leader: bool,
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
}

impl RollupMetadataModel {
    pub const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get(cluster_id: &ClusterId) -> Result<Self, DbError> {
        let key = (Self::ID, cluster_id);
        database()?.get(&key)
    }

    pub fn get_mut(cluster_id: &ClusterId) -> Result<Lock<'static, Self>, DbError> {
        let key = (Self::ID, cluster_id);
        database()?.get_mut(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (Self::ID, self.cluster_id.clone());
        database()?.put(&key, self)
    }
}
