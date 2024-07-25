mod encrypted_transaction;
mod raw_transaction;

pub use encrypted_transaction::*;
pub use raw_transaction::*;

use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Transaction {
    Raw(RawTransaction),
    Encrypted(EncryptedTransaction),
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub enum TransactionType {
//     Eth = "eth_tx",
//     EthBundle = "eth_bundle_tx",
//     Undefined = "undefined",
// }
