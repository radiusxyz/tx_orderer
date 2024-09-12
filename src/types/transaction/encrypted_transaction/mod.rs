use ethers::types as eth_types;

use crate::types::prelude::*;

mod eth_bundle_transaction;
mod eth_transaction;

pub use eth_bundle_transaction::*;
pub use eth_transaction::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OpenData {
    Eth(EthOpenData),
    EthBundle(EthBundleOpenData),
}

impl From<EthOpenData> for OpenData {
    fn from(open_data: EthOpenData) -> Self {
        OpenData::Eth(open_data)
    }
}

impl From<EthBundleOpenData> for OpenData {
    fn from(open_data: EthBundleOpenData) -> Self {
        OpenData::EthBundle(open_data)
    }
}

impl OpenData {
    pub fn to_raw_transaction(&self, encrypted_data: &EncryptData) -> Transaction {
        match (self, encrypted_data) {
            (OpenData::Eth(open_data), EncryptData::Eth(encrypt_data)) => {
                Transaction::Eth(open_data.to_raw_transaction(encrypt_data))
            }
            // tODO(jaemin): impl EthBundle
            (OpenData::EthBundle(_open_data), EncryptData::EthBundle(_encrypt_data)) => {
                Transaction::EthBundle
            }
            _ => panic!("Invalid combination of OpenData and EncryptData"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EncryptData {
    Eth(EthEncryptData),
    EthBundle(EthBundleEncryptData),
}

// TODO(jaemin): Add Ethbundle
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Transaction {
    Eth(eth_types::Transaction),
    EthBundle,
}

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

    pub fn open_data(&self) -> OpenData {
        match self {
            EncryptedTransaction::Eth(eth) => OpenData::from(eth.open_data().clone()),
            EncryptedTransaction::EthBundle(eth_bundle) => {
                OpenData::from(eth_bundle.open_data().clone())
            }
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

    pub fn raw_transaction_hash(&self) -> &RawTransactionHash {
        match self {
            EncryptedTransaction::Eth(eth) => eth.open_data().raw_tx_hash(),
            EncryptedTransaction::EthBundle(eth_bundle) => eth_bundle.open_data().raw_tx_hash(),
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
pub struct EncryptedTransactionList(Vec<EncryptedTransaction>);

impl EncryptedTransactionList {
    pub fn new(value: Vec<EncryptedTransaction>) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> Vec<EncryptedTransaction> {
        self.0
    }
}
