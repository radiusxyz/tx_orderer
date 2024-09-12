use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedTransactionModel;

impl EncryptedTransactionModel {
    pub const ID: &'static str = stringify!(EncryptedTransactionModel);

    pub fn put(
        rollup_id: &String,
        rollup_block_height: u64,
        transaction_order: u64,

        encrypted_transaction: EncryptedTransaction,
        time_lock_puzzle: Option<TimeLockPuzzle>,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, rollup_id, rollup_block_height, transaction_order);

        kvstore()?.put(key, &(encrypted_transaction, time_lock_puzzle))
    }

    pub fn get(
        rollup_id: &String,
        block_height: u64,
        transaction_order: u64,
    ) -> Result<(EncryptedTransaction, Option<TimeLockPuzzle>), KvStoreError> {
        let key = &(Self::ID, rollup_id, block_height, transaction_order);

        kvstore()?.get(key)
    }
}
