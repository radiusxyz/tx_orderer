use ethers::{
    types as eth_types,
    utils::rlp::{self, Decodable, DecoderError},
};
use ssal::avs::types::{hex, Bytes};

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthTransaction {
    open_data: EthOpenData,
    encrypted_data: EncryptedData,
    pvde_zkp: Option<PvdeZkp>,
}

impl EthTransaction {
    pub fn new(
        open_data: EthOpenData,
        encrypted_data: EncryptedData,
        pvde_zkp: Option<PvdeZkp>,
    ) -> Self {
        Self {
            open_data,
            encrypted_data,
            pvde_zkp,
        }
    }

    pub fn open_data(&self) -> &EthOpenData {
        &self.open_data
    }

    pub fn encrypted_data(&self) -> &EncryptedData {
        &self.encrypted_data
    }

    pub fn pvde_zkp(&self) -> Option<&PvdeZkp> {
        self.pvde_zkp.as_ref()
    }

    pub fn mut_pvde_zkp(&mut self) -> Option<&mut PvdeZkp> {
        self.pvde_zkp.as_mut()
    }
}

// TODO: stompesi
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EthOpenData {
    pub raw_tx_hash: RawTxHash,
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
            raw_tx_hash: RawTxHash::new(format!("0x{}", hex::encode(transaction.hash.as_bytes()))),
            from: transaction.from,
            nonce: transaction.nonce.into(),
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
    pub fn to_raw_transaction(&self, payload: &EthEncryptData) -> eth_types::Transaction {
        eth_types::Transaction {
            hash: eth_types::H256::from_slice(
                hex::decode(self.raw_tx_hash.clone().into_inner())
                    .unwrap()
                    .as_slice(),
            ),
            nonce: self.nonce.clone().into(),
            block_hash: self.block_hash.clone(),
            block_number: self.block_number.clone(),
            transaction_index: self.transaction_index.clone(),
            from: self.from.clone(),
            gas_price: self.gas_price.clone(),
            gas: self.gas_limit.clone(),
            to: payload.to.clone(),
            value: payload.value.clone(),
            input: payload.input.clone(),
            v: self.signature.v.clone().into(),
            r: self.signature.r.clone(),
            s: self.signature.s.clone(),
            transaction_type: self.transaction_type.clone(),
            access_list: self.access_list.clone(),
            max_priority_fee_per_gas: self.max_priority_fee_per_gas.clone(),
            max_fee_per_gas: self.max_fee_per_gas.clone(),
            chain_id: self.chain_id.clone(),
            other: self.other.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EthEncryptData {
    pub to: Option<eth_types::Address>,
    pub value: eth_types::U256,
    #[serde(rename = "data")]
    pub input: eth_types::Bytes,
}

pub fn to_raw_tx(transaction: eth_types::Transaction) -> String {
    let rlp_bytes = transaction.rlp();
    format!("0x{}", hex::encode(rlp_bytes))
}

pub fn eth_bytes_to_hex(bytes: Bytes) -> String {
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
