use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RollupMetadata {
    block_height: BlockHeight,
    transaction_order: TransactionOrder,
}

impl RollupMetadata {
    pub fn new(block_height: BlockHeight, transaction_order: TransactionOrder) -> Self {
        Self {
            block_height,
            transaction_order,
        }
    }

    pub fn transaction_order(&self) -> TransactionOrder {
        self.transaction_order.clone()
    }

    pub fn increment_transaction_order(&mut self) {
        self.transaction_order.increment();
    }

    pub fn block_height(&self) -> BlockHeight {
        self.block_height
    }
}
