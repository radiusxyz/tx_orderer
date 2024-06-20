use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadata {
    pub ssal_block_number: u64,
    pub rollup_block_number: u64,
    pub sequencer_list: SequencerList,
    pub is_leader: bool,
}

impl ClusterMetadata {
    const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get(database: &Database) -> Result<Self, database::Error> {
        database.get(&Self::ID)
    }

    pub fn put(&self, database: &Database) -> Result<(), database::Error> {
        database.put(&Self::ID, self)
    }

    pub fn new(
        ssal_block_number: u64,
        rollup_block_number: u64,
        sequencer_list: SequencerList,
        is_leader: bool,
    ) -> Self {
        Self {
            ssal_block_number,
            rollup_block_number,
            sequencer_list,
            is_leader,
        }
    }
}
