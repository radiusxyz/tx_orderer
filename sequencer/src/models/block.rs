use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockModel {
    rollup_id: RollupId,
    block: Block,
}
impl BlockModel {
    pub fn new(rollup_id: RollupId, block: Block) -> Self {
        Self { rollup_id, block }
    }
}

impl BlockModel {
    const ID: &'static str = stringify!(RollupBlockModel);

    pub fn get(rollup_id: &RollupId, rollup_block_height: &BlockHeight) -> Result<Self, DbError> {
        let key = (Self::ID, rollup_id, rollup_block_height);
        database()?.get(&key)
    }

    pub fn put(&self) -> Result<(), DbError> {
        let key = (Self::ID, &self.rollup_id, self.block.block_height());
        database()?.put(&key, self)
    }
}
