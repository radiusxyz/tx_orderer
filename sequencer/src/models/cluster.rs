use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadataModel {
    pub ssal_block_height: BlockHeight,
    pub rollup_block_height: BlockHeight,

    pub transaction_order: TransactionOrder,

    pub is_leader: bool,
}

impl Default for ClusterMetadataModel {
    fn default() -> Self {
        Self {
            ssal_block_height: BlockHeight::new(0),
            rollup_block_height: BlockHeight::new(0),
            transaction_order: TransactionOrder::new(0),
            is_leader: false,
        }
    }
}

impl ClusterMetadataModel {
    pub const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get() -> Result<Self, database::Error> {
        database()?.get(&Self::ID)
    }

    pub fn get_mut() -> Result<Lock<'static, Self>, database::Error> {
        database()?.get_mut(&Self::ID)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        database()?.put(&Self::ID, self)
    }

    pub fn new(
        ssal_block_height: BlockHeight,
        rollup_block_height: BlockHeight,
        transaction_order: TransactionOrder,
        is_leader: bool,
    ) -> Self {
        Self {
            ssal_block_height,
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
