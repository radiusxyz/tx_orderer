use ethers::types as eth_types;

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthEncryptedBundleTransaction {
    open_data: EthBundleOpenData,
    encrypted_transaction: EncryptedData,
    pvde_zkp: Option<PvdeZkp>,
}

impl EthEncryptedBundleTransaction {
    pub fn open_data(&self) -> &EthBundleOpenData {
        &self.open_data
    }

    pub fn encrypted_data(&self) -> &EncryptedData {
        &self.encrypted_transaction
    }

    pub fn pvde_zkp(&self) -> Option<&PvdeZkp> {
        self.pvde_zkp.as_ref()
    }

    pub fn update_pvde_zkp(&mut self, pvde_zkp: Option<PvdeZkp>) {
        self.pvde_zkp = pvde_zkp;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EthBundleOpenData {
    pub raw_tx_hash: RawTransactionHash,
}

impl EthBundleOpenData {
    pub fn raw_tx_hash(&self) -> &RawTransactionHash {
        &self.raw_tx_hash
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EthBundleEncryptData {
    pub to: Option<eth_types::Address>,
    pub value: eth_types::U256,

    #[serde(rename = "data")]
    pub input: eth_types::Bytes,
}
