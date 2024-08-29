mod encrypted_transaction;
mod raw_transaction;

pub use encrypted_transaction::*;
pub use raw_transaction::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TransactionModel {
    Raw(RawTransactionModel),
    Encrypted(EncryptedTransactionModel),
}
