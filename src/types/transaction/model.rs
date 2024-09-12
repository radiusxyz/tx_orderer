use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedTransactionModel {
    encrypted_transaction: EncryptedTransaction,
    time_lock_puzzle: Option<TimeLockPuzzle>,
}

impl EncryptedTransactionModel {
    pub fn new(
        encrypted_transaction: EncryptedTransaction,
        time_lock_puzzle: Option<TimeLockPuzzle>,
    ) -> Self {
        Self {
            encrypted_transaction,
            time_lock_puzzle,
        }
    }

    pub fn encrypted_transaction(&self) -> &EncryptedTransaction {
        &self.encrypted_transaction
    }

    pub fn time_lock_puzzle(&self) -> &Option<TimeLockPuzzle> {
        &self.time_lock_puzzle
    }
}

impl EncryptedTransactionModel {
    pub const ID: &'static str = stringify!(EncryptedTransactionModel);

    pub fn put(
        &self,
        rollup_id: &String,
        block_height: u64,
        transaction_order: u64,
    ) -> Result<(), KvStoreError> {
        let key = (Self::ID, rollup_id, block_height, transaction_order);
        kvstore()?.put(&key, self)
    }

    pub fn get(
        rollup_id: &String,
        block_height: u64,
        transaction_order: u64,
    ) -> Result<Self, KvStoreError> {
        let key = (Self::ID, rollup_id, block_height, transaction_order);
        kvstore()?.get(&key)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawTransactionModel {
    raw_transaction: RawTransaction,
}

impl RawTransactionModel {
    pub fn new(raw_transaction: RawTransaction) -> Self {
        Self { raw_transaction }
    }

    pub fn raw_transaction(&self) -> &RawTransaction {
        &self.raw_transaction
    }
}

impl RawTransactionModel {
    pub const ID: &'static str = stringify!(RawTransactionModel);

    pub fn put(
        &self,
        rollup_id: &String,
        block_height: u64,
        transaction_order: u64,
    ) -> Result<(), KvStoreError> {
        let key = (Self::ID, rollup_id, block_height, transaction_order);
        kvstore()?.put(&key, self)
    }

    pub fn get(
        rollup_id: &String,
        block_height: u64,
        transaction_order: u64,
    ) -> Result<Self, KvStoreError> {
        let key = (Self::ID, rollup_id, block_height, transaction_order);
        kvstore()?.get(&key)
    }
}
