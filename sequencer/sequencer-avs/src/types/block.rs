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
pub struct BlockKey(&'static str, SsalBlockNumber, RollupBlockNumber);

impl BlockKey {
    pub const IDENTIFIER: &'static str = stringify!(BlockKey);

    pub fn new(ssal_block_number: SsalBlockNumber, rollup_block_number: RollupBlockNumber) -> Self {
        Self(Self::IDENTIFIER, ssal_block_number, rollup_block_number)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Block {
    transaction_order: TransactionOrder,
}

impl Block {
    pub const ID: &'static str = stringify!(BlockKey);

    pub fn new() -> Self {
        Self {
            transaction_order: TransactionOrder::from(0),
        }
    }

    pub fn get(
        ssal_block_number: SsalBlockNumber,
        rollup_block_number: RollupBlockNumber,
    ) -> Result<Self, Error> {
        let key = (Self::ID, ssal_block_number, rollup_block_number);
        Ok(database().get(&key)?)
    }

    pub fn transaction_order(&mut self) -> TransactionOrder {
        self.transaction_order += 1;
        self.transaction_order
    }
}
