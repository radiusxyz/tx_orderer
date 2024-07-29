use std::fmt;

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionOrder(u64);

impl TransactionOrder {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl From<u64> for TransactionOrder {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<TransactionOrder> for u64 {
    fn from(transaction_order: TransactionOrder) -> Self {
        transaction_order.0
    }
}

impl PartialEq for TransactionOrder {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl fmt::Display for TransactionOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderHash(String);

impl Default for OrderHash {
    fn default() -> Self {
        Self("".to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderHashList(Vec<OrderHash>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitmentData {
    pub rollup_id: RollupId,
    pub block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
    pub previous_order_hash: OrderHash,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitment {
    pub data: OrderCommitmentData,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitmentList(Vec<OrderCommitment>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BundleOrderCommitment {
    order_commitment_list: OrderCommitmentList,
    signature: Signature,
}
