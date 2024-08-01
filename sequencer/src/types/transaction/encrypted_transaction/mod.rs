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

impl EncryptedTransaction {
    pub fn encrypted_data(&self) -> &EncryptedData {
        match self {
            EncryptedTransaction::Eth(eth) => eth.encrypted_data(),
            EncryptedTransaction::EthBundle(eth_bundle) => eth_bundle.encrypted_data(),
        }
    }
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
