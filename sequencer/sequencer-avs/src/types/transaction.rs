use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction(usize);

impl From<usize> for Transaction {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TransactionOrder(u64);

impl From<u64> for TransactionOrder {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl std::ops::AddAssign<u64> for TransactionOrder {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
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
