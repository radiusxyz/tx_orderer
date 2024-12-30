mod bundle_order_commitment;
mod order_commitment_type;
mod single_order_commitment;

pub use bundle_order_commitment::*;
pub use order_commitment_type::*;
use radius_sdk::kvstore::Model;
use serde::{Deserialize, Serialize};
pub use single_order_commitment::*;

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &String, rollup_block_height: u64, transaction_order: u64))]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum OrderCommitment {
    Single(SingleOrderCommitment),
    Bundle(BundleOrderCommitment),
}
