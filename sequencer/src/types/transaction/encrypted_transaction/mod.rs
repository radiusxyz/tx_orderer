use crate::types::prelude::*;

mod eth_bundle_transaction;
mod eth_transaction;

pub use eth_bundle_transaction::*;
pub use eth_transaction::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EncryptedTransaction {
    Eth(EthEncryptedTransaction),
    EthBundle(EthEncryptedBundleTransaction),
}

impl EncryptedTransaction {
    pub fn encrypted_data(&self) -> &EncryptedData {
        match self {
            EncryptedTransaction::Eth(eth) => eth.encrypted_data(),
            EncryptedTransaction::EthBundle(eth_bundle) => eth_bundle.encrypted_data(),
        }
    }

    pub fn open_data(&self) -> &EthOpenData {
        match self {
            EncryptedTransaction::Eth(eth) => eth.open_data(),
            _ => unreachable!(),
        }
    }

    pub fn pvde_zkp(&self) -> Option<&PvdeZkp> {
        match self {
            EncryptedTransaction::Eth(eth) => eth.pvde_zkp(),
            EncryptedTransaction::EthBundle(eth_bundle) => eth_bundle.pvde_zkp(),
        }
    }

    pub fn update_pvde_zkp(&mut self, pvde_zkp: Option<PvdeZkp>) {
        match self {
            EncryptedTransaction::Eth(eth) => eth.update_pvde_zkp(pvde_zkp),
            EncryptedTransaction::EthBundle(eth_bundle) => eth_bundle.update_pvde_zkp(pvde_zkp),
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

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedTransactionList(Vec<Option<EncryptedTransaction>>);
