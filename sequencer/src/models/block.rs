use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockModel {
    pub block: Block,
}

impl BlockModel {
    const ID: &'static str = stringify!(RollupBlockModel);

    pub fn get(rollup_id: &RollupId, rollup_block_height: &BlockHeight) -> Result<Self, DbError> {
        let key = (Self::ID, rollup_id, rollup_block_height);
        database()?.get(&key)
    }

    pub fn put(&self, rollup_id: &RollupId) -> Result<(), DbError> {
        let key = (Self::ID, rollup_id, self.block.block_height());
        database()?.put(&key, self)
    }
}
