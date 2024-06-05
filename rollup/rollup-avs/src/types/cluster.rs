use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cluster {
    leader: (PublicKey, Option<RpcAddress>),
    followers: Vec<(PublicKey, Option<RpcAddress>)>,
}

impl Cluster {
    const ID: &'static str = stringify!(Cluster);

    pub fn get(rollup_block_number: RollupBlockNumber) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number);
        database().get(&key)
    }

    pub fn put(&self, rollup_block_number: RollupBlockNumber) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_block_number);
        database().put(&key, self)
    }

    pub fn new(
        leader: (PublicKey, Option<RpcAddress>),
        followers: Vec<(PublicKey, Option<RpcAddress>)>,
    ) -> Self {
        Self { leader, followers }
    }

    pub fn leader(&self) -> &(PublicKey, Option<RpcAddress>) {
        &self.leader
    }

    pub fn followers(&self) -> &Vec<(PublicKey, Option<RpcAddress>)> {
        &self.followers
    }
}
