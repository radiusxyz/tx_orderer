use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RollupMetadata {
    block_height: BlockHeight,
    transaction_order: TransactionOrder,
    order_hash: OrderHash,
}

impl RollupMetadata {
    pub fn new(
        block_height: BlockHeight,
        transaction_order: TransactionOrder,
        order_hash: OrderHash,
    ) -> Self {
        Self {
            block_height,
            transaction_order,
            order_hash,
        }
    }

    pub fn transaction_order(&self) -> TransactionOrder {
        self.transaction_order.clone()
    }

    pub fn increase_transaction_order(&mut self) {
        self.transaction_order.increase();
    }

    pub fn block_height(&self) -> BlockHeight {
        self.block_height
    }

    pub fn order_hash(&self) -> &OrderHash {
        &self.order_hash
    }

    pub fn update_order_hash(&mut self, order_hash: OrderHash) {
        self.order_hash = order_hash;
    }
}
