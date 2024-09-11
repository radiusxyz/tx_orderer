use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitmentModel;

impl OrderCommitmentModel {
    const ID: &'static str = stringify!(OrderCommitmentModel);

    pub fn put(
        rollup_id: &String,
        rollup_block_height: u64,
        transaction_order: u64,
        order_commitment: &OrderCommitment,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, rollup_id, rollup_block_height, transaction_order);

        kvstore()?.put(key, order_commitment)
    }

    pub fn get(
        rollup_id: &String,
        rollup_block_height: u64,
        transaction_order: u64,
    ) -> Result<OrderCommitment, KvStoreError> {
        let key = &(Self::ID, rollup_id, rollup_block_height, transaction_order);

        kvstore()?.get(key)
    }
}
