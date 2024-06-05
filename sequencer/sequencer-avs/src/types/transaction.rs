use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction(usize);

impl From<usize> for Transaction {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessedTransaction {
    order_commitment: OrderCommitment,
    transaction: Transaction,
}

impl ProcessedTransaction {
    const ID: &'static str = stringify!(ProcessedTransaction);

    pub fn get(order_commitment: OrderCommitment) -> Result<Self, database::Error> {
        let key = (
            Self::ID,
            order_commitment.rollup_block_number,
            order_commitment.transaction_order,
        );
        database().get(&key)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        let key = (
            Self::ID,
            self.order_commitment.rollup_block_number,
            self.order_commitment.transaction_order,
        );
        database().put(&key, self)
    }

    pub fn new(order_commitment: OrderCommitment, transaction: Transaction) -> Self {
        Self {
            order_commitment,
            transaction,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitment {
    rollup_block_number: RollupBlockNumber,
    transaction_order: u64,
}

impl OrderCommitment {
    pub fn new(rollup_block_number: RollupBlockNumber, transaction_order: u64) -> Self {
        Self {
            rollup_block_number,
            transaction_order,
        }
    }
}
