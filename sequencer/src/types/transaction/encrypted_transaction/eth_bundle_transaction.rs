use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthBundleTransaction {
    open_data: OpenData,
    encrypted_transaction: EncryptedData,
    pvde_zkp: Option<PvdeZkp>,
}

// TODO: stompesi
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OpenData {
    pub raw_tx_hash: String,
}

impl EthBundleTransaction {
    pub fn opend_data(&self) -> &OpenData {
        &self.open_data
    }

    pub fn encrypted_data(&self) -> &EncryptedData {
        &self.encrypted_transaction
    }

    pub fn pvde_zkp(&self) -> Option<&PvdeZkp> {
        self.pvde_zkp.as_ref()
    }

    pub fn mut_pvde_zkp(&mut self) -> Option<&mut PvdeZkp> {
        self.pvde_zkp.as_mut()
    }
}
