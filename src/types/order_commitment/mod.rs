mod model;

use ethers::utils::hex;
pub use model::*;
use sha3::{Digest, Sha3_256};

use crate::types::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct OrderHash(String);

impl OrderHash {
    pub fn issue_order_hash(&self, raw_tx_hash: &RawTransactionHash) -> OrderHash {
        let mut hasher = Sha3_256::new();

        // TODO(jaemin): check hasher params
        hasher.update(self.0.as_bytes());
        hasher.update(raw_tx_hash.clone().into_inner().as_bytes());

        let order_hash_bytes = hasher.finalize();
        OrderHash(hex::encode(order_hash_bytes))
    }
}

impl Default for OrderHash {
    fn default() -> Self {
        Self("0000000000000000000000000000000000000000000000000000000000000000".to_owned())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderHashList(Vec<OrderHash>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitmentData {
    pub rollup_id: String,
    pub block_height: u64,
    pub transaction_order: u64,
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
