use super::prelude::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct RollupBlockNumber(u64);

impl std::ops::Rem<usize> for RollupBlockNumber {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        self.0 as usize % rhs
    }
}

impl std::ops::AddAssign<u64> for RollupBlockNumber {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl std::ops::Sub<u64> for RollupBlockNumber {
    type Output = Self;

    fn sub(self, rhs: u64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl From<u64> for RollupBlockNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl RollupBlockNumber {
    const ID: &'static str = stringify!(RollupBlockNumber);

    pub fn get() -> Result<Self, database::Error> {
        database().get(&Self::ID)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        database().put(&Self::ID, self)
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

impl std::ops::SubAssign<u64> for SsalBlockNumber {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 -= rhs;
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
        database().get(&Self::ID)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        database().put(&Self::ID, self)
    }
}
