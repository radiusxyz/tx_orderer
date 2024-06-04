use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClusterMetadata {
    ssal_block_number: SsalBlockNumber,
    rollup_block_number: RollupBlockNumber,
    leader_index: u64,
    sequencer_list: SequencerList,
}

impl ClusterMetadata {
    const ID: &'static str = stringify!(ClusterMetadata);

    pub fn get() -> Result<Self, Error> {
        database().get(&Self::ID)
    }

    pub fn put(&self) -> Result<(), Error> {
        database().put(&Self::ID, self)
    }

    pub fn new(
        ssal_block_number: SsalBlockNumber,
        rollup_block_number: RollupBlockNumber,
        sequencer_list: SequencerList,
    ) -> Self {
        Self {
            ssal_block_number,
            rollup_block_number,
            leader_index: rollup_block_number % sequencer_list.len(),
            sequencer_list,
        }
    }

    pub fn ssal_block_number(&self) -> SsalBlockNumber {
        self.ssal_block_number
    }

    pub fn rollup_block_number(&self) -> RollupBlockNumber {
        self.rollup_block_number
    }

    /// TODO:
    pub fn is_leader(&self) -> bool {
        false
    }

    /// TODO:
    pub fn leader_address(&self) -> Option<SequencerAddress> {
        None
    }

    pub fn leader(&self) -> Option<&(PublicKey, Option<SequencerAddress>)> {
        self.sequencer_list.get_by_index(self.leader_index)
    }

    pub fn sequencer_list(&self) -> &SequencerList {
        &self.sequencer_list
    }
}
