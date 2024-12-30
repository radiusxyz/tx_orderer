use serde::{Deserialize, Serialize};

use crate::types::{deserialize_merkle_path, serialize_merkle_path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignOrderCommitment {
    pub data: OrderCommitmentData,
    pub signature: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]

pub struct OrderCommitmentData {
    pub rollup_id: String,
    pub block_height: u64,
    pub transaction_order: u64,

    #[serde(serialize_with = "serialize_merkle_path")]
    #[serde(deserialize_with = "deserialize_merkle_path")]
    pub pre_merkle_path: Vec<[u8; 32]>,
}
