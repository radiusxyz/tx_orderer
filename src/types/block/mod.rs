mod model;

pub use model::*;

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timestamp(String);

impl Timestamp {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlockCommitment(Vec<u8>);

impl AsRef<[u8]> for BlockCommitment {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for BlockCommitment {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub block_height: u64,

    pub encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
    pub raw_transaction_list: Vec<RawTransaction>,

    pub proposer_address: Address,
    pub signature: Signature,

    pub block_commitment: BlockCommitment,
}

impl Block {
    pub fn new(
        block_height: u64,
        encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
        raw_transaction_list: Vec<RawTransaction>,
        proposer_address: Address,
        signature: Signature,
        block_commitment: BlockCommitment,
    ) -> Self {
        Self {
            block_height,
            encrypted_transaction_list,
            raw_transaction_list,
            proposer_address,
            signature,
            block_commitment,
        }
    }

    pub fn block_height(&self) -> u64 {
        self.block_height
    }
}
