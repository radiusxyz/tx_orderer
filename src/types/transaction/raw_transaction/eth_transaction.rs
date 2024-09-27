use const_hex::{hex, ToHex, ToHexExt};

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthRawTransaction(pub String);

impl From<String> for EthRawTransaction {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl EthRawTransaction {
    pub fn raw_transaction_hash(&self) -> RawTransactionHash {
        println!("self.0: {:?}", self.0);

        // TODO: decode_rlp_transaction
        let decoded_transaction = decode_rlp_transaction(&self.0).unwrap();

        let transaction_hash = const_hex::encode_prefixed(decoded_transaction.hash);

        RawTransactionHash::from(transaction_hash)
    }
}
