mod batch_commitment;
pub use batch_commitment::*;

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &RollupId, batch_number: u64))]
pub struct Batch {
    pub batch_number: u64,

    pub encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
    pub raw_transaction_list: Vec<RawTransaction>,

    pub batch_commitment: BatchCommitment,
    pub batch_creator_address: Address,

    pub signature: Signature,
}

impl Batch {
    pub fn new(
        batch_number: u64,
        encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
        raw_transaction_list: Vec<RawTransaction>,
        batch_commitment: BatchCommitment,
        batch_creator_address: Address,
        signature: Signature,
    ) -> Self {
        Self {
            batch_number,
            encrypted_transaction_list,
            raw_transaction_list,
            batch_commitment,
            batch_creator_address,
            signature,
        }
    }
}
