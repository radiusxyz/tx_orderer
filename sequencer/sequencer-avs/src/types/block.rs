use std::str::FromStr;

use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupBlock(Vec<UserTransaction>);

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

    pub fn push(&mut self, transaction: UserTransaction) {
        self.0.push(transaction)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlockCommitment(String);

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
