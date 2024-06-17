use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SequencerStatus {
    Uninitialized,
    Running,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerList(Vec<(H160, Option<String>)>);

impl From<(Vec<H160>, Vec<Option<String>>)> for SequencerList {
    fn from(value: (Vec<H160>, Vec<Option<String>>)) -> Self {
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

    pub fn new(public_key_list: Vec<H160>, address_list: Vec<Option<String>>) -> Self {
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

    pub fn iter(&self) -> core::slice::Iter<(H160, Option<String>)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> std::vec::IntoIter<(H160, Option<String>)> {
        self.0.into_iter()
    }

    pub fn split_leader_from_followers(
        self,
        leader_index: usize,
    ) -> ((H160, Option<String>), Vec<(H160, Option<String>)>) {
        let mut inner = self.into_inner();
        let leader = inner.remove(leader_index);
        (leader, inner)
    }

    fn into_inner(self) -> Vec<(H160, Option<String>)> {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Me(H160);

impl std::cmp::PartialEq<H160> for Me {
    fn eq(&self, other: &H160) -> bool {
        &self.0 == other
    }
}

impl From<H160> for Me {
    fn from(value: H160) -> Self {
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

    pub fn as_public_key(&self) -> &H160 {
        &self.0
    }

    pub fn into_public_key(self) -> H160 {
        self.0
    }
}
