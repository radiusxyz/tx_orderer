mod block_commitment;
pub use block_commitment::*;

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str, block_height: u64))]
pub struct Block {
    pub block_height: u64,

    pub encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
    pub raw_transaction_list: Vec<RawTransaction>,

    pub signature: Signature,

    pub block_commitment: BlockCommitment,
    pub block_creator_address: Address,
}

impl Block {
    pub fn new(
        block_height: u64,
        encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
        raw_transaction_list: Vec<RawTransaction>,
        signature: Signature,
        block_commitment: BlockCommitment,
        block_creator_address: Address,
    ) -> Self {
        Self {
            block_height,
            encrypted_transaction_list,
            raw_transaction_list,
            signature,
            block_commitment,
            block_creator_address,
        }
    }
}
