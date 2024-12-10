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
pub struct BlockCommitment(String);

impl Default for BlockCommitment {
    fn default() -> Self {
        Self(const_hex::encode_prefixed([0; 32]))
    }
}

impl From<[u8; 32]> for BlockCommitment {
    fn from(value: [u8; 32]) -> Self {
        Self(const_hex::encode_prefixed(value))
    }
}

impl From<OrderHash> for BlockCommitment {
    fn from(value: OrderHash) -> Self {
        Self(value.into_inner())
    }
}

impl From<&str> for BlockCommitment {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for BlockCommitment {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl BlockCommitment {
    pub fn as_bytes(self) -> Result<Vec<u8>, const_hex::FromHexError> {
        const_hex::decode(self.0)
    }

    pub fn as_hex_string(self) -> String {
        self.0
    }
}

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

    pub fn block_height(&self) -> u64 {
        self.block_height
    }
}
