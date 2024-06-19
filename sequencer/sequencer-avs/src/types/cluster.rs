use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadata {
    ssal_block_number: SsalBlockNumber,
    rollup_block_number: RollupBlockNumber,
    leader: (Address, Option<String>),
    followers: Vec<(Address, Option<String>)>,
    is_leader: bool,
}

impl ClusterMetadata {
    const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get() -> Result<Self, database::Error> {
        database()?.get(&Self::ID)
    }

    pub fn put(&self) -> Result<(), database::Error> {
        database()?.put(&Self::ID, self)
    }

    pub fn new(
        ssal_block_number: SsalBlockNumber,
        rollup_block_number: RollupBlockNumber,
        leader: (Address, Option<String>),
        followers: Vec<(Address, Option<String>)>,
        is_leader: bool,
    ) -> Self {
        Self {
            ssal_block_number,
            rollup_block_number,
            leader,
            followers,
            is_leader,
        }
    }

    pub fn ssal_block_number(&self) -> SsalBlockNumber {
        self.ssal_block_number
    }

    pub fn rollup_block_number(&self) -> RollupBlockNumber {
        self.rollup_block_number
    }

    pub fn leader(&self) -> &(Address, Option<String>) {
        &self.leader
    }

    pub fn into_leader(self) -> (Address, Option<String>) {
        self.leader
    }

    pub fn followers(&self) -> &Vec<(Address, Option<String>)> {
        &self.followers
    }

    pub fn into_followers(self) -> Vec<(Address, Option<String>)> {
        self.followers
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader
    }
}
