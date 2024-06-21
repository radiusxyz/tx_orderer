use block_commitment::get_block_commitment;

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

    pub fn commitment(&self, seed: [u8; 32]) -> BlockCommitment {
        get_block_commitment(&self.0, seed).into()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockCommitment(Vec<u8>);

impl From<Vec<u8>> for BlockCommitment {
    fn from(value: Vec<u8>) -> Self {
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
}
