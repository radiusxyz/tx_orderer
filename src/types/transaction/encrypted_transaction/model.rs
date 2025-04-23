use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedTransactionModel;

impl EncryptedTransactionModel {
    pub const ID: &'static str = stringify!(EncryptedTransactionModel);

    pub fn put(
        rollup_id: &RollupId,
        batch_number: u64,
        transaction_order: u64,
        encrypted_transaction: &EncryptedTransaction,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, rollup_id, batch_number, transaction_order);

        kvstore()?.put(key, encrypted_transaction)
    }

    pub fn put_with_transaction_hash(
        rollup_id: &RollupId,
        transaction_hash: &RawTransactionHash,
        encrypted_transaction: &EncryptedTransaction,
    ) -> Result<(), KvStoreError> {
        let key = &(Self::ID, rollup_id, transaction_hash);

        kvstore()?.put(key, encrypted_transaction)
    }

    pub fn get(
        rollup_id: &RollupId,
        batch_number: u64,
        transaction_order: u64,
    ) -> Result<EncryptedTransaction, KvStoreError> {
        let key = &(Self::ID, rollup_id, batch_number, transaction_order);

        kvstore()?.get(key)
    }

    pub fn get_with_transaction_hash(
        rollup_id: &RollupId,
        transaction_hash: &str,
    ) -> Result<EncryptedTransaction, KvStoreError> {
        let key = &(Self::ID, rollup_id, transaction_hash);

        kvstore()?.get(key)
    }

    pub fn get_mut(
        rollup_id: &RollupId,
        batch_number: u64,
        transaction_order: u64,
    ) -> Result<Lock<'static, EncryptedTransaction>, KvStoreError> {
        let key = &(Self::ID, rollup_id, batch_number, transaction_order);

        kvstore()?.get_mut(key)
    }
}
