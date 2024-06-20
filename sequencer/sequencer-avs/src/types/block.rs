use block_commitment::get_block_commitment;

use super::prelude::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct BlockMetadata {
    block_height: u64,
}

impl Default for BlockMetadata {
    fn default() -> Self {
        Self { block_height: 1 }
    }
}

impl BlockMetadata {
    const ID: &'static str = stringify!(BlockMetadata);

    pub fn get(database: &Database, rollup_block_number: u64) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number);
        database.get(&key)
    }

    pub fn put(
        &self,
        database: &Database,
        rollup_block_number: u64,
    ) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_block_number);
        database.put(&key, self)
    }

    pub fn issue_order_commitment(
        database: &Database,
        rollup_block_number: u64,
    ) -> Result<OrderCommitment, database::Error> {
        let key = (Self::ID, rollup_block_number);
        let mut block_metadata: Lock<Self> = database.get_mut(&key)?;
        block_metadata.block_height += 1;

        let order_commitment =
            OrderCommitment::new(rollup_block_number, block_metadata.block_height);

        block_metadata.commit()?;

        Ok(order_commitment)
    }

    pub fn block_height(&self) -> u64 {
        self.block_height
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupBlock(Vec<UserTransaction>);

impl RollupBlock {
    const ID: &'static str = stringify!(RollupBlock);

    pub fn get(database: &Database, rollup_block_number: u64) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number);
        database.get(&key)
    }

    pub fn put(
        &self,
        database: &Database,
        rollup_block_number: u64,
    ) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_block_number);
        database.put(&key, self)
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

    pub fn get(database: &Database, rollup_block_number: u64) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number);
        database.get(&key)
    }

    pub fn put(
        &self,
        database: &Database,
        rollup_block_number: u64,
    ) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_block_number);
        database.put(&key, self)
    }
}
