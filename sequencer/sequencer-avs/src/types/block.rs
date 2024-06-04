use super::prelude::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct RollupBlockNumber(u64);

impl std::ops::Add<u64> for RollupBlockNumber {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::AddAssign<u64> for RollupBlockNumber {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl std::ops::Rem<u64> for RollupBlockNumber {
    type Output = u64;

    fn rem(self, rhs: u64) -> Self::Output {
        self.0 & rhs
    }
}

impl From<u64> for RollupBlockNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SsalBlockNumber(u64);

impl From<u64> for SsalBlockNumber {
    fn from(value: u64) -> Self {
        Self(value)
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

    pub fn get(rollup_block_number: RollupBlockNumber) -> Result<Self, Error> {
        let key = (Self::ID, rollup_block_number);
        database().get(&key)
    }

    pub fn put(&self, rollup_block_number: RollupBlockNumber) -> Result<(), Error> {
        let key = (Self::ID, rollup_block_number);
        database().put(&key, self)
    }

    pub fn issue_order_commitment(
        rollup_block_number: RollupBlockNumber,
    ) -> Result<OrderCommitment, Error> {
        let key = (Self::ID, rollup_block_number);
        let mut block_metadata: Lock<Self> = database().get_mut(&key)?;
        block_metadata.block_height += 1;
        let order_commitment =
            OrderCommitment::new(rollup_block_number, block_metadata.block_height);
        block_metadata.commit()?;
        Ok(order_commitment)
    }
}
