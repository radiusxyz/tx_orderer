use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionOrder(u64);

impl TransactionOrder {
    pub fn new(value: u64) -> Self {
        TransactionOrder(value)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderHash(String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderHashList(Vec<OrderHash>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitmentData {
    rollup_id: RollupId,
    block_height: BlockHeight,
    transaction_order: TransactionOrder,
    previous_order_hash: OrderHash,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitment {
    order_commitment_data: OrderCommitmentData,
    signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitmentList(Vec<OrderCommitment>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BundleOrderCommitment {
    order_commitment_list: OrderCommitmentList,
    signature: Signature,
}
