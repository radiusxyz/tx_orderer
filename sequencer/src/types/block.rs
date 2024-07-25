use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockHeight(u64);

impl BlockHeight {
    pub fn new(value: u64) -> Self {
        BlockHeight(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::ops::Sub<u64> for BlockHeight {
    type Output = u64;

    fn sub(self, rhs: u64) -> Self::Output {
        self.0 - rhs
    }
}

impl From<u64> for BlockHeight {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timestamp(String);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlockCommitment(Vec<u8>);

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
