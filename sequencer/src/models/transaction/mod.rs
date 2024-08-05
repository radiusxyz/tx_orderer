mod encrypted_transaction;
mod raw_transaction;

pub use encrypted_transaction::*;
pub use raw_transaction::*;
use serde::{Deserialize, Serialize};

use crate::models::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TransactionModel {
    Raw(RawTransactionModel),
    Encrypted(EncryptedTransactionModel),
}

impl TransactionModel {
    pub fn put(
        &self,
        rollup_id: &ClusterId,
        block_height: &BlockHeight,
        transaction_order: &TransactionOrder,
    ) -> Result<(), DbError> {
        match self {
            Self::Raw(raw) => raw.put(rollup_id, block_height, transaction_order),
            Self::Encrypted(encrypted) => encrypted.put(rollup_id, block_height, transaction_order),
        }
    }

    pub fn get(
        rollup_id: &ClusterId,
        block_height: &BlockHeight,
        transaction_order: &TransactionOrder,
    ) -> Result<Self, DbError> {
        RawTransactionModel::get(rollup_id, block_height, transaction_order)
            .map(Self::Raw)
            .or_else(|error| {
                if error.is_none_type() {
                    EncryptedTransactionModel::get(rollup_id, block_height, transaction_order)
                        .map(Self::Encrypted)
                } else {
                    Err(error)
                }
            })
    }
}
