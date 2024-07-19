use super::prelude::*;

pub const BLOCK_MARGIN: u64 = 7;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SsalBlockNumber(u64);

impl std::ops::Sub<u64> for SsalBlockNumber {
    type Output = u64;

    fn sub(self, rhs: u64) -> Self::Output {
        self.0 - rhs
    }
}

impl From<u64> for SsalBlockNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl SsalBlockNumber {
    pub const ID: &'static str = stringify!(SsalBlockNumber);

    pub fn get() -> Result<Self, database::Error> {
        database()?.get(&Self::ID)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        database()?.put(&Self::ID, self)
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupBlock(Vec<UserTransaction>);

impl From<Vec<UserTransaction>> for RollupBlock {
    fn from(value: Vec<UserTransaction>) -> Self {
        Self(value)
    }
}

impl RollupBlock {
    const ID: &'static str = stringify!(RollupBlock);

    pub fn get(rollup_id: u32, rollup_block_number: u64) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_id, rollup_block_number);
        database()?.get(&key)
    }

    pub fn put(&self, rollup_id: u32, rollup_block_number: u64) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_id, rollup_block_number);
        database()?.put(&key, self)
    }

    pub fn new(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
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

impl BlockCommitment {
    pub const ID: &'static str = stringify!(BlockCommitment);

    pub fn get(rollup_id: u32, rollup_block_number: u64) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_id, rollup_block_number);
        database()?.get(&key)
    }

    pub fn put(&self, rollup_id: u32, rollup_block_number: u64) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_id, rollup_block_number);
        database()?.put(&key, self)
    }

    pub fn to_bytes(&self) -> Bytes {
        Bytes::from_iter(&self.0)
    }
}
