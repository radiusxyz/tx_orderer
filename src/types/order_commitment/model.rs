use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitmentModel;

impl OrderCommitmentModel {
    const ID: &'static str = stringify!(OrderCommitmentModel);

    pub fn get(
        rollup_id: &String,
        rollup_block_height: u64,
    ) -> Result<OrderCommitmentData, KvStoreError> {
        let key = &(Self::ID, rollup_id, rollup_block_height);

        kvstore()?.get(key)
    }

    pub fn put(
        rollup_id: &String,
        rollup_block_height: u64,
        order_commitment_data: &OrderCommitmentData,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, rollup_id, rollup_block_height);

        kvstore()?.put(key, order_commitment_data)
    }
}
