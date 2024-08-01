use crate::types::prelude::*;

mod eth_bundle_transaction;
mod eth_transaction;

pub use eth_bundle_transaction::*;
pub use eth_transaction::*;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct RawTxHash(String);

impl RawTxHash {
    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RawTransaction {
    Eth(EthTransaction),
    EthBundle(EthBundleTransaction),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawTransactionList(Vec<RawTransaction>);
