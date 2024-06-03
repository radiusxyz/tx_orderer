use serde::{Deserialize, Serialize};
use ssal::ethereum::PublicKey;

use crate::types::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ClusterKey(&'static str, SsalBlockNumber);

impl ClusterKey {
    const IDENTIFIER: &'static str = stringify!(ClusterKey);

    pub fn new(ssal_block_number: SsalBlockNumber) -> Self {
        Self(Self::IDENTIFIER, ssal_block_number)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cluster {
    ssal_block_number: SsalBlockNumber,
    rollup_block_number: RollupBlockNumber,
    leader_index: usize,
    sequencer_list: SequencerList,
}

impl Cluster {
    pub fn new(
        ssal_block_number: SsalBlockNumber,
        rollup_block_number: RollupBlockNumber,
        sequencer_list: SequencerList,
    ) -> Self {
        Self {
            ssal_block_number,
            rollup_block_number,
            leader_index: ,
            sequencer_list,
        }
    }

    pub fn ssal_block_number(&self) -> SsalBlockNumber {
        self.ssal_block_number
    }

    pub fn rollup_block_number(&self) -> RollupBlockNumber {
        self.rollup_block_number
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClusterStatus {
    Uninitialized,
    Initialized(OrderCommitment),
}
