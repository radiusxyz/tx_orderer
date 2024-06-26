use std::str::FromStr;

use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupBlock(Vec<UserTransaction>);

impl From<Vec<UserTransaction>> for RollupBlock {
    fn from(value: Vec<UserTransaction>) -> Self {
        Self(value)
    }
}

impl RollupBlock {
    const ID: &'static str = stringify!(RollupBlock);

    pub fn get(rollup_block_number: u64) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number);
        database()?.get(&key)
    }

    pub fn put(&self, rollup_block_number: u64) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_block_number);
        database()?.put(&key, self)
    }

    pub fn new(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlockCommitment(String);

impl AsRef<str> for BlockCommitment {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for BlockCommitment {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl BlockCommitment {
    const ID: &'static str = stringify!(BlockCommitment);

    pub fn get(rollup_block_number: u64) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number);
        database()?.get(&key)
    }

    pub fn put(&self, rollup_block_number: u64) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_block_number);
        database()?.put(&key, self)
    }

    pub fn to_bytes(&self) -> Result<Bytes, ssal::avs::Error> {
        Bytes::from_str(self.0.as_str())
            .map_err(|error| (ssal::avs::ErrorKind::ParseBlockCommitment, error).into())
    }
}
