use crate::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockHeight(usize);

impl std::cmp::Eq for BlockHeight {}

impl std::cmp::PartialEq<usize> for BlockHeight {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl std::cmp::PartialEq<Self> for BlockHeight {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::cmp::PartialEq<&Self> for BlockHeight {
    fn eq(&self, other: &&Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Display for BlockHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add<usize> for BlockHeight {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::Sub<usize> for BlockHeight {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Default for BlockHeight {
    fn default() -> Self {
        Self(1)
    }
}

impl From<usize> for BlockHeight {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl BlockHeight {
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn value(&self) -> usize {
        self.0
    }
}
