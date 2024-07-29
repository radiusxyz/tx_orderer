use crate::types::prelude::*;

pub type BlockHeight = u64;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timestamp(String);

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
    block_height: BlockHeight,

    encrypted_transaction_list: EncryptedTransactionList,
    raw_transaction_list: RawTransactionList,

    proposer_address: Address,
    signature: Signature,

    timestamp: Timestamp,

    block_commitment: BlockCommitment,
}
