// TODO:
use ethers::{
    types::{self as eth_types, Bytes, U256},
    utils::{
        hex,
        rlp::{self, Decodable, DecoderError},
    },
};
use serde_json::Value;

use crate::{error::Error, types::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthTransactionData {
    encrypted_data: EncryptedData,
    open_data: EthOpenData,

    plain_data: Option<EthPlainData>,
}

impl EthTransactionData {
    pub fn encrypted_data(&self) -> &EncryptedData {
        &self.encrypted_data
    }

    pub fn open_data(&self) -> &EthOpenData {
        &self.open_data
    }

    pub fn plain_data(&self) -> Option<&EthPlainData> {
        self.plain_data.as_ref()
    }

    pub fn update_plain_data(&mut self, plain_data: EthPlainData) {
        self.plain_data = Some(plain_data);
    }

    pub fn convert_to_rollup_transaction(&self) -> Result<RollupTransaction, Error> {
        if self.plain_data.is_none() {
            return Err(Error::NotExistPlainData);
        }

        Ok(RollupTransaction::Eth(
            self.open_data
                .convert_to_rollup_transaction(self.plain_data.as_ref().unwrap()),
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EthOpenData {
    pub raw_tx_hash: RawTransactionHash,
    pub from: eth_types::Address,
    pub nonce: eth_types::U256,
    pub gas_price: Option<eth_types::U256>,
    pub gas_limit: eth_types::U256,
    pub signature: eth_types::Signature, // (v, r, s)

    // Additional fields
    pub block_hash: Option<eth_types::H256>,
    pub block_number: Option<eth_types::U64>,
    pub transaction_index: Option<eth_types::U64>,
    pub transaction_type: Option<eth_types::U64>,
    pub access_list: Option<eth_types::transaction::eip2930::AccessList>,
    pub max_priority_fee_per_gas: Option<eth_types::U256>,
    pub max_fee_per_gas: Option<eth_types::U256>,
    pub chain_id: Option<eth_types::U256>,
    pub other: eth_types::OtherFields,
}

impl From<eth_types::Transaction> for EthOpenData {
    fn from(transaction: eth_types::Transaction) -> Self {
        Self {
            raw_tx_hash: RawTransactionHash::new(format!(
                "0x{}",
                hex::encode(transaction.hash.as_bytes())
            )),
            from: transaction.from,
            nonce: transaction.nonce,
            gas_price: transaction.gas_price,
            gas_limit: transaction.gas,
            signature: eth_types::Signature {
                r: transaction.r,
                s: transaction.s,
                v: transaction.v.as_u64(),
            },
            block_hash: transaction.block_hash,
            block_number: transaction.block_number,
            transaction_index: transaction.transaction_index,
            transaction_type: transaction.transaction_type,
            access_list: transaction.access_list,
            max_priority_fee_per_gas: transaction.max_priority_fee_per_gas,
            max_fee_per_gas: transaction.max_fee_per_gas,
            chain_id: transaction.chain_id,
            other: transaction.other,
        }
    }
}

impl EthOpenData {
    pub fn convert_to_rollup_transaction(
        &self,
        plain_data: &EthPlainData,
    ) -> eth_types::Transaction {
        eth_types::Transaction {
            hash: eth_types::H256::from_slice(
                hex::decode(self.raw_tx_hash.clone().into_inner())
                    .unwrap()
                    .as_slice(),
            ),
            nonce: self.nonce,
            block_hash: self.block_hash,
            block_number: self.block_number,
            transaction_index: self.transaction_index,
            from: self.from,
            gas_price: self.gas_price,
            gas: self.gas_limit,
            to: plain_data.to,
            value: plain_data.value,
            input: plain_data.input.clone(),
            v: self.signature.v.into(),
            r: self.signature.r,
            s: self.signature.s,
            transaction_type: self.transaction_type,
            access_list: self.access_list.clone(),
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas,
            chain_id: self.chain_id,
            other: self.other.clone(),
        }
    }

    pub fn raw_tx_hash(&self) -> &RawTransactionHash {
        &self.raw_tx_hash
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EthPlainData {
    pub to: Option<eth_types::Address>,
    pub value: eth_types::U256,
    #[serde(rename = "data")]
    pub input: eth_types::Bytes,
}

pub fn to_raw_tx(transaction: eth_types::Transaction) -> String {
    let rlp_bytes = transaction.rlp();
    format!("0x{}", hex::encode(rlp_bytes))
}

pub fn eth_bytes_to_hex(bytes: eth_types::Bytes) -> String {
    format!("0x{}", hex::encode(bytes))
}

pub fn decode_transaction(rlp: &rlp::Rlp) -> Result<eth_types::Transaction, DecoderError> {
    eth_types::Transaction::decode(rlp)
}

pub fn decode_rlp_transaction(rlp_hex: &str) -> Result<eth_types::Transaction, DecoderError> {
    let hex_str = rlp_hex.trim_start_matches("0x");
    let rlp_bytes = hex::decode(hex_str).map_err(|_| DecoderError::Custom("hex decode error"))?;
    let rlp = rlp::Rlp::new(&rlp_bytes);

    eth_types::Transaction::decode(&rlp)
}

pub fn to_encrypt_data_string(eth_transaction: &eth_types::Transaction) -> String {
    let payload = serde_json::json!({
        "to": eth_transaction.to,
        "value": eth_transaction.value,
        "data": eth_transaction.input,
    });
    serde_json::to_string(&payload).unwrap()
}

pub fn string_to_eth_plain_data(string: &str) -> Result<EthPlainData, Box<dyn std::error::Error>> {
    let json: Value = serde_json::from_str(string)?;

    let to = if let Some(to_str) = json.get("to").and_then(|v| v.as_str()) {
        Some(to_str.parse::<eth_types::Address>()?)
    } else {
        None
    };

    let value = if let Some(value_str) = json.get("value").and_then(|v| v.as_str()) {
        U256::from_dec_str(value_str)?
    } else {
        U256::zero()
    };

    let input = if let Some(data_str) = json.get("data").and_then(|v| v.as_str()) {
        Bytes::from(hex::decode(data_str.trim_start_matches("0x"))?)
    } else {
        Bytes::default()
    };

    Ok(EthPlainData { to, value, input })
}
