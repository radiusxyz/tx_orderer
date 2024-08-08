use crate::types::prelude::{Deserialize, Serialize};

mod eth_bundle_transaction;
mod eth_transaction;

pub use eth_bundle_transaction::EthRawBundleTransaction;
pub use eth_transaction::EthRawTransaction;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct RawTransactionHash(String);

impl RawTransactionHash {
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
    EthBundle(EthRawBundleTransaction),
}

impl From<EthRawTransaction> for RawTransaction {
    fn from(raw_transaction: EthRawTransaction) -> Self {
        RawTransaction::Eth(raw_transaction)
    }
}

impl From<EthRawBundleTransaction> for RawTransaction {
    fn from(raw_transaction: EthRawBundleTransaction) -> Self {
        RawTransaction::EthBundle(raw_transaction)
    }
}

impl RawTransaction {
    pub fn raw_transaction_hash(&self) -> RawTransactionHash {
        match self {
            RawTransaction::Eth(eth) => eth.raw_transaction_hash(),
            RawTransaction::EthBundle(eth_bundle) => eth_bundle.raw_transaction_hash(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RawTransactionList(Vec<RawTransaction>);

impl RawTransactionList {
    pub fn new(value: Vec<RawTransaction>) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> Vec<RawTransaction> {
        self.0
    }
}
