use crate::types::prelude::*;

pub type RollupId = String;
pub type RollupIdList = Vec<RollupId>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rollup {
    rollup_id: RollupId,
    block_height: BlockHeight,
}

impl Rollup {
    pub fn new(rollup_id: RollupId) -> Self {
        Self {
            rollup_id,
            block_height: BlockHeight::default(),
        }
    }

    pub fn rollup_id(&self) -> &RollupId {
        &self.rollup_id
    }

    pub fn block_height(&self) -> &BlockHeight {
        &self.block_height
    }
}
