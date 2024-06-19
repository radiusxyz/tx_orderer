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

impl SequencerList {
    const ID: &'static str = stringify!(SequencerList);

    pub fn get(ssal_block_number: SsalBlockNumber) -> Result<Self, database::Error> {
        let key = (Self::ID, ssal_block_number);
        database()?.get(&key)
    }

    pub fn put(&self, ssal_block_number: SsalBlockNumber) -> Result<(), database::Error> {
        let key = (Self::ID, ssal_block_number);
        database()?.put(&key, self)
    }

    pub fn new(public_key_list: Vec<Address>, address_list: Vec<Option<String>>) -> Self {
        Self(
            public_key_list
                .into_iter()
                .zip(address_list.into_iter())
                .collect(),
        )
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> core::slice::Iter<(Address, Option<String>)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> std::vec::IntoIter<(Address, Option<String>)> {
        self.0.into_iter()
    }

    pub fn split_leader_from_followers(
        self,
        leader_index: usize,
    ) -> ((Address, Option<String>), Vec<(Address, Option<String>)>) {
        let mut inner = self.into_inner();
        let leader = inner.remove(leader_index);
        (leader, inner)
    }

    fn into_inner(self) -> Vec<(Address, Option<String>)> {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Me(Address);

impl std::cmp::PartialEq<Address> for Me {
    fn eq(&self, other: &Address) -> bool {
        &self.0 == other
    }
}

impl From<Address> for Me {
    fn from(value: Address) -> Self {
        Self(value)
    }
}

impl Me {
    const ID: &'static str = stringify!(Me);

    pub fn get() -> Result<Self, database::Error> {
        database()?.get(&Self::ID)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        database()?.put(&Self::ID, self)
    }

    pub fn as_public_key(&self) -> &Address {
        &self.0
    }

    pub fn into_public_key(self) -> Address {
        self.0
    }
}
