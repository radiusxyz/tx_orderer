use crate::types::prelude::*;

mod eth_bundle_transaction;
mod eth_transaction;

pub use eth_bundle_transaction::*;
pub use eth_transaction::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawTxHash(String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RawTransaction {
    Eth(EthTransaction),
    EthBundle(EthBundleTransaction),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawTransactionList(Vec<RawTransaction>);
