use radius_sdk::signature::{Address, ChainType, Signature};
use serde::{Deserialize, Serialize};

use crate::types::{deserialize_merkle_path, serialize_merkle_path, RawTransactionHash, RollupId};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignOrderCommitment {
    pub data: OrderCommitmentData,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitmentData {
    pub rollup_id: RollupId,
    pub batch_number: u64,
    pub transaction_order: u64,

    pub transaction_hash: String,

    #[serde(
        serialize_with = "serialize_merkle_path",
        deserialize_with = "deserialize_merkle_path"
    )]
    pub pre_merkle_path: Vec<[u8; 32]>,
}

impl Default for OrderCommitmentData {
    fn default() -> Self {
        Self {
            rollup_id: RollupId::new(),
            batch_number: 0,
            transaction_order: 0,
            transaction_hash: RawTransactionHash::default().as_string(),
            pre_merkle_path: Vec::new(),
        }
    }
}

impl SignOrderCommitment {
    pub fn get_signer_address(&self, chain_type: ChainType) -> Address {
        match self.signature.get_signer_address(chain_type, &self.data) {
            Ok(address) => address,
            Err(_) => Address::default(),
        }
    }
}
