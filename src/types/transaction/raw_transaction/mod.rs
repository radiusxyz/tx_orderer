use crate::types::prelude::{Deserialize, Serialize};

mod eth_bundle_transaction;
mod eth_transaction;
mod model;

pub use eth_bundle_transaction::*;
pub use eth_transaction::*;
pub use model::*;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct RawTransactionHash(String);

impl Default for RawTransactionHash {
    fn default() -> Self {
        Self(const_hex::encode([0; 32]))
    }
}

impl From<String> for RawTransactionHash {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<[u8]> for RawTransactionHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<str> for RawTransactionHash {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl RawTransactionHash {
    pub fn new(value: impl AsRef<[u8]>) -> Self {
        Self(const_hex::encode(value))
    }

    pub fn as_string(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
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
