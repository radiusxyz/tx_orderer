use super::prelude::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SequencerStatus {
    Uninitialized,
    Running,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerList(Vec<(PublicKey, Option<RpcAddress>)>);

impl SequencerList {
    const ID: &'static str = stringify!(SequencerList);

    pub fn get(ssal_block_number: SsalBlockNumber) -> Result<Self, database::Error> {
        let key = (Self::ID, ssal_block_number);
        database().get(&key)
    }

    pub fn put(&self, ssal_block_number: SsalBlockNumber) -> Result<(), database::Error> {
        let key = (Self::ID, ssal_block_number);
        database().put(&key, self)
    }

    pub fn new(public_key_list: Vec<PublicKey>, address_list: Vec<Option<RpcAddress>>) -> Self {
        Self(std::iter::zip(public_key_list, address_list).collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> core::slice::Iter<(PublicKey, Option<RpcAddress>)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> std::vec::IntoIter<(PublicKey, Option<RpcAddress>)> {
        self.0.into_iter()
    }

    pub fn split_leader_from_followers(
        self,
        leader_index: usize,
    ) -> (
        (PublicKey, Option<RpcAddress>),
        Vec<(PublicKey, Option<RpcAddress>)>,
    ) {
        let mut inner = self.into_inner();
        let leader = inner.remove(leader_index);
        (leader, inner)
    }

    fn into_inner(self) -> Vec<(PublicKey, Option<RpcAddress>)> {
        self.0
    }
}
