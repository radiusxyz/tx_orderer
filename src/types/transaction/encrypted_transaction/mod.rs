use ethers::types as eth_types;

use crate::{error::Error, types::prelude::*};

mod eth_bundle_transaction;
mod eth_transaction;
mod model;

pub use eth_bundle_transaction::*;
pub use eth_transaction::*;
pub use model::*;

#[test]
fn works() {
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(tag = "type", content = "encrypted_transaction")]
    #[serde(rename_all = "snake_case")]
    pub enum Transaction {
        Pvde(Pvde),
        Skde(Skde),
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Pvde {
        data: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Skde {
        key_id: String,
        data: String,
    }

    let pvde = serde_json::json!({
        "type": "pvde",
        "encrypted_transaction": {
            "data": "PVDE data"
        }
    });
    println!("{:?}", serde_json::from_value::<Transaction>(pvde).unwrap());

    let skde = serde_json::json!({
        "type": "skde",
        "encrypted_transaction": {
            "key_id": "key ID",
            "data": "SKDE data"
        }
    });
    println!("{:?}", serde_json::from_value::<Transaction>(skde).unwrap());
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum EncryptedTransaction {
    Pvde(PvdeEncryptedTransaction),
    Skde(SkdeEncryptedTransaction),
}

impl EncryptedTransaction {
    pub fn raw_transaction_hash(&self) -> RawTransactionHash {
        // TODO:
        RawTransactionHash::new("hi")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PvdeEncryptedTransaction {
    transaction_data: TransactionData,

    time_lock_puzzle: TimeLockPuzzle,
    pvde_zkp: Option<PvdeZkp>,
}

impl PvdeEncryptedTransaction {
    pub fn transaction_data(&self) -> &TransactionData {
        &self.transaction_data
    }

    pub fn time_lock_puzzle(&self) -> &TimeLockPuzzle {
        &self.time_lock_puzzle
    }

    pub fn pvde_zkp(&self) -> Option<&PvdeZkp> {
        self.pvde_zkp.as_ref()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkdeEncryptedTransaction {
    transaction_data: TransactionData,

    key_id: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum TransactionData {
    Eth(EthTransactionData),
    EthBundle(EthBundleTransactionData),
}

impl TransactionData {
    pub fn convert_to_rollup_transaction(&self) -> Result<RollupTransaction, Error> {
        match self {
            Self::Eth(data) => data.convert_to_rollup_transaction(),
            Self::EthBundle(data) => data.convert_to_rollup_transaction(),
        }
    }
}

/////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupTransaction {
    Eth(eth_types::Transaction),
    EthBundle,
}

/////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedData(String);

impl AsRef<[u8]> for EncryptedData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
