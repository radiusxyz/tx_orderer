use crate::types::prelude::*;

mod eth_bundle_transaction;
mod eth_transaction;

pub use eth_bundle_transaction::*;
pub use eth_transaction::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EncryptedTransaction {
    Eth(EthTransaction),
    EthBundle(EthBundleTransaction),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedData(String);

impl AsRef<[u8]> for EncryptedData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl EncryptedData {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedTransactionList(Vec<Option<EncryptedTransaction>>);
