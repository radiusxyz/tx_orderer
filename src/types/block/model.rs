use crate::types::prelude::*;

pub struct BlockModel;

impl BlockModel {
    const ID: &'static str = stringify!(BlockModel);

    pub fn get(rollup_id: &String, rollup_block_height: u64) -> Result<Block, KvStoreError> {
        let key = &(Self::ID, rollup_id, rollup_block_height);

        kvstore()?.get(key)
    }

    pub fn put(
        rollup_id: &String,
        rollup_block_height: u64,
        block: &Block,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, rollup_id, rollup_block_height);

        kvstore()?.put(key, block)
    }
}
