use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupBlockModel {
    pub rollup_block: Block,
}

impl RollupBlockModel {
    const ID: &'static str = stringify!(RollupBlockModel);

    pub fn get(
        rollup_id: &RollupId,
        rollup_block_height: BlockHeight,
    ) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_id, rollup_block_height);
        database()?.get(&key)
    }

    pub fn put(
        &self,
        rollup_id: &RollupId,
        rollup_block_height: BlockHeight,
    ) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_id, rollup_block_height);
        database()?.put(&key, self)
    }
}
