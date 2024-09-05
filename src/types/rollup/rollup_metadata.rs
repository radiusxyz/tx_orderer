use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RollupMetadata {
    block_height: u64,
    transaction_order: TransactionOrder,
    order_hash: OrderHash,
}

impl RollupMetadata {
    pub fn new(
        block_height: u64,
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
        self.transaction_order
    }

    pub fn increase_transaction_order(&mut self) {
        self.transaction_order.increase();
    }

    pub fn block_height(&self) -> u64 {
        self.block_height
    }

    pub fn order_hash(&self) -> &OrderHash {
        &self.order_hash
    }

    pub fn update_order_hash(&mut self, order_hash: OrderHash) {
        self.order_hash = order_hash;
    }

    pub fn issue_new_block(&mut self) {
        self.block_height += 1;
        self.transaction_order = 0.into();
        self.order_hash = OrderHash::new();
    }
}
