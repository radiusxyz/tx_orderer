use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SequencerStatus {
    Uninitialized,
    Running,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerList(Vec<(Address, Option<String>)>);

impl From<(Vec<Address>, Vec<Option<String>>)> for SequencerList {
    fn from(value: (Vec<Address>, Vec<Option<String>>)) -> Self {
        Self(std::iter::zip(value.0, value.1).collect())
    }
}

impl From<Vec<(Address, Option<String>)>> for SequencerList {
    fn from(value: Vec<(Address, Option<String>)>) -> Self {
        Self(value)
    }
}

impl SequencerList {
    const ID: &'static str = stringify!(SequencerList);

    pub fn get(ssal_block_number: u64) -> Result<Self, database::Error> {
        let key = (Self::ID, ssal_block_number);
        database()?.get(&key)
    }

    pub fn put(&self, ssal_block_number: u64) -> Result<(), database::Error> {
        let key = (Self::ID, ssal_block_number);
        database()?.put(&key, self)
    }

    pub fn delete(&self, ssal_block_number: u64) -> Result<(), database::Error> {
        let key = (Self::ID, ssal_block_number);
        database()?.delete(&key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn into_inner(self) -> Vec<(Address, Option<String>)> {
        self.0
    }
}
