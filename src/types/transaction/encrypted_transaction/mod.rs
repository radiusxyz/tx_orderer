use ethers_core::types as eth_types;

use crate::{error::Error, types::prelude::*};

mod eth_bundle_transaction;
mod eth_transaction;
mod model;

pub use eth_bundle_transaction::*;
pub use eth_transaction::*;
pub use model::*;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum EncryptedTransaction {
    Pvde(PvdeEncryptedTransaction),
    Skde(SkdeEncryptedTransaction),
}

impl EncryptedTransaction {
    pub fn raw_transaction_hash(&self) -> RawTransactionHash {
        RawTransactionHash::new("hi")
    }

    pub fn update_transaction_data(&mut self, transaction_data: TransactionData) {
        match self {
            Self::Pvde(pvde) => {
                pvde.transaction_data = transaction_data;
            }
            Self::Skde(skde) => {
                skde.transaction_data = transaction_data;
            }
        }
    }

    pub fn transaction_data(&self) -> &TransactionData {
        match self {
            Self::Pvde(pvde_encrypted_transaction) => pvde_encrypted_transaction.transaction_data(),
            Self::Skde(skde_encrypted_transaction) => skde_encrypted_transaction.transaction_data(),
        }
    }

    pub fn encrypted_data(&self) -> &EncryptedData {
        match self {
            Self::Pvde(pvde_encrypted_transaction) => pvde_encrypted_transaction
                .transaction_data()
                .encrypted_data(),
            Self::Skde(skde_encrypted_transaction) => skde_encrypted_transaction
                .transaction_data()
                .encrypted_data(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PvdeEncryptedTransaction {
    transaction_data: TransactionData,

    time_lock_puzzle: TimeLockPuzzle,
    pvde_zkp: Option<PvdeZkp>,
}

impl PvdeEncryptedTransaction {
    pub fn new(
        transaction_data: TransactionData,
        time_lock_puzzle: TimeLockPuzzle,
        pvde_zkp: Option<PvdeZkp>,
    ) -> Self {
        Self {
            transaction_data,
            time_lock_puzzle,
            pvde_zkp,
        }
    }

    pub fn transaction_data(&self) -> &TransactionData {
        &self.transaction_data
    }

    pub fn time_lock_puzzle(&self) -> &TimeLockPuzzle {
        &self.time_lock_puzzle
    }

    pub fn pvde_zkp(&self) -> Option<&PvdeZkp> {
        self.pvde_zkp.as_ref()
    }

    pub fn set_pvde_zkp(&mut self, pvde_zkp: PvdeZkp) {
        self.pvde_zkp = Some(pvde_zkp);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkdeEncryptedTransaction {
    transaction_data: TransactionData,
    key_id: u64,
}

impl SkdeEncryptedTransaction {
    pub fn new(transaction_data: TransactionData, key_id: u64) -> Self {
        Self {
            transaction_data,
            key_id,
        }
    }

    pub fn transaction_data(&self) -> &TransactionData {
        &self.transaction_data
    }

    pub fn mut_transaction_data(&mut self) -> &mut TransactionData {
        &mut self.transaction_data
    }

    pub fn key_id(&self) -> u64 {
        self.key_id
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum TransactionData {
    Eth(EthTransactionData),
    EthBundle(EthBundleTransactionData),
}

impl From<EthTransactionData> for TransactionData {
    fn from(value: EthTransactionData) -> Self {
        Self::Eth(value)
    }
}

impl From<EthBundleTransactionData> for TransactionData {
    fn from(value: EthBundleTransactionData) -> Self {
        Self::EthBundle(value)
    }
}

impl TransactionData {
    pub fn convert_to_rollup_transaction(&self) -> Result<RollupTransaction, Error> {
        match self {
            Self::Eth(data) => data.convert_to_rollup_transaction(),
            Self::EthBundle(data) => data.convert_to_rollup_transaction(),
        }
    }

    pub fn update_plain_data(&mut self, plain_data: EthPlainData) {
        match self {
            Self::Eth(data) => {
                data.plain_data = Some(plain_data);
            }
            Self::EthBundle(_data) => {
                unimplemented!()
            }
        }
    }

    pub fn encrypted_data(&self) -> &EncryptedData {
        match self {
            Self::Eth(data) => data.encrypted_data(),
            Self::EthBundle(data) => data.encrypted_data(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum PlainData {
    Eth(EthPlainData),
    EthBundle(EthBundlePlainData),
}

impl From<EthPlainData> for PlainData {
    fn from(value: EthPlainData) -> Self {
        Self::Eth(value)
    }
}

impl From<EthBundlePlainData> for PlainData {
    fn from(value: EthBundlePlainData) -> Self {
        Self::EthBundle(value)
    }
}

/////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupTransaction {
    Eth(eth_types::Transaction),
    EthBundle,
}

impl RollupTransaction {
    pub fn to_raw_transaction(&self) -> Result<RawTransaction, Error> {
        match self {
            Self::Eth(transaction) => {
                let raw_transaction_string =
                    serde_json::to_string(transaction).map_err(Error::Deserialize)?;

                Ok(RawTransaction::Eth(EthRawTransaction::from(
                    raw_transaction_string,
                )))
            }
            // Todo: implement EthBundle
            Self::EthBundle => Ok(RawTransaction::EthBundle(EthRawBundleTransaction::from(
                String::new(),
            ))),
        }
    }
}

/////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedData(String);

impl AsRef<str> for EncryptedData {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<[u8]> for EncryptedData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl EncryptedData {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for EncryptedData {
    fn from(value: String) -> Self {
        Self(value)
    }
}
