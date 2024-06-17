use block_commitment::get_block_commitment;

use super::prelude::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct RollupBlockNumber(u64);

impl std::ops::Add<u64> for RollupBlockNumber {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::Sub<u64> for RollupBlockNumber {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl std::ops::AddAssign<u64> for RollupBlockNumber {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl std::ops::Rem<usize> for RollupBlockNumber {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        self.0 as usize % rhs
    }
}

impl From<u64> for RollupBlockNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SsalBlockNumber(u64);

impl std::ops::Sub<u64> for SsalBlockNumber {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl From<u64> for SsalBlockNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl SsalBlockNumber {
    const ID: &'static str = stringify!(SsalBlockNumber);

    pub fn get() -> Result<Self, database::Error> {
        database()?.get(&Self::ID)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        database()?.put(&Self::ID, self)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct BlockMetadata {
    block_height: u64,
}

impl Default for BlockMetadata {
    fn default() -> Self {
        Self { block_height: 0 }
    }
}

impl BlockMetadata {
    const ID: &'static str = stringify!(BlockMetadata);

    pub fn get(rollup_block_number: RollupBlockNumber) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number);
        database()?.get(&key)
    }

    pub fn put(&self, rollup_block_number: RollupBlockNumber) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_block_number);
        database()?.put(&key, self)
    }

    pub fn issue_order_commitment(
        rollup_block_number: RollupBlockNumber,
    ) -> Result<OrderCommitment, database::Error> {
        let key = (Self::ID, rollup_block_number);
        let mut block_metadata: Lock<Self> = database()?.get_mut(&key)?;
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

    pub fn get(rollup_block_number: RollupBlockNumber) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number);
        database()?.get(&key)
    }

    pub fn put(&self, rollup_block_number: RollupBlockNumber) -> Result<(), database::Error> {
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

    pub fn get(rollup_block_number: RollupBlockNumber) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number);
        database()?.get(&key)
    }

    pub fn put(&self, rollup_block_number: RollupBlockNumber) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_block_number);
        database()?.put(&key, self)
    }
}
