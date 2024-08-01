use crate::types::prelude::*;

mod eth_bundle_transaction;
mod eth_transaction;

pub use eth_bundle_transaction::*;
pub use eth_transaction::*;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct RawTxHash(String);

impl RawTxHash {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RawTransaction {
    Eth(EthRawTransaction),
    EthBundle(EthBundleRawTransaction),
}

impl From<EthRawTransaction> for RawTransaction {
    fn from(raw_transaction: EthRawTransaction) -> Self {
        RawTransaction::Eth(raw_transaction)
    }
}

impl From<EthBundleRawTransaction> for RawTransaction {
    fn from(raw_transaction: EthBundleRawTransaction) -> Self {
        RawTransaction::EthBundle(raw_transaction)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawTransactionList(Vec<RawTransaction>);
