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
pub struct BlockCommitment(pub String);

impl Default for BlockCommitment {
    fn default() -> Self {
        Self(const_hex::encode([0; 32]))
    }
}

impl AsRef<[u8]> for BlockCommitment {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<str> for BlockCommitment {
    fn as_ref(&self) -> &str {
        self.0.as_str()
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
    pub is_leader: bool,
}

impl Block {
    pub fn new(
        block_height: u64,
        encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
        raw_transaction_list: Vec<RawTransaction>,
        proposer_address: Address,
        signature: Signature,
        block_commitment: BlockCommitment,
        is_leader: bool,
    ) -> Self {
        Self {
            block_height,
            encrypted_transaction_list,
            raw_transaction_list,
            proposer_address,
            signature,
            block_commitment,
            is_leader,
        }
    }

    pub fn block_height(&self) -> u64 {
        self.block_height
    }
}
