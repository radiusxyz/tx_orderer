use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthBundleTransaction {
    open_data: OpenData,
    encrypted_transaction: EncryptedData,
}

// TODO: stompesi
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OpenData {
    pub raw_tx_hash: String,
}

impl EthBundleTransaction {
    pub fn encrypted_data(&self) -> &EncryptedData {
        &self.encrypted_transaction
    }
}
