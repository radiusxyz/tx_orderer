use ethers_core::types as eth_types;

use crate::{error::Error, types::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthRawBundleTransaction(pub String);

impl From<String> for EthRawBundleTransaction {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl EthRawBundleTransaction {
    pub fn raw_transaction_hash(&self) -> RawTransactionHash {
        let raw_transaction_string = serde_json::to_string(&self.0).unwrap();
        let parsed_raw_transaction_string: String =
            serde_json::from_str(&raw_transaction_string).unwrap();
        let decoded_transaction = decode_rlp_transaction(&parsed_raw_transaction_string).unwrap();

        RawTransactionHash::new(const_hex::encode_prefixed(
            decoded_transaction.hash.as_bytes(),
        ))
    }

    pub fn rollup_transaction(&self) -> Result<eth_types::Transaction, Error> {
        let raw_transaction_string = serde_json::to_string(&self.0).unwrap();
        let parsed_raw_transaction_string: String =
            serde_json::from_str(&raw_transaction_string).unwrap();
        decode_rlp_transaction(&parsed_raw_transaction_string)
            .map_err(|_| Error::InvalidTransaction)
    }
}
