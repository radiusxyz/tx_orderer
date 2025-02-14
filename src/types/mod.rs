mod block;
mod cluster;
mod config;
mod liveness;
mod merkle;
mod order_commitment;
mod platform;
mod rollup;
mod transaction;
mod validation;
mod version;

pub use block::*;
pub use cluster::*;
pub use config::*;
pub use liveness::*;
pub use merkle::*;
pub use order_commitment::*;
pub use platform::*;
use radius_sdk::signature::Address;
pub use rollup::*;
use serde::{ser::SerializeSeq, Deserialize, Deserializer};
pub use transaction::*;
pub use validation::*;
pub use version::*;

pub(crate) mod prelude {
    pub use radius_sdk::{
        kvstore::{kvstore, KvStoreError, Lock, Model},
        signature::{Address, Signature},
    };
    pub use serde::{Deserialize, Serialize};

    pub use crate::types::*;
}

pub fn serialize_address<S>(address: &Address, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&address.as_hex_string())
}

pub fn serialize_address_list<S>(addresses: &[Address], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(addresses.len()))?;
    for address in addresses {
        seq.serialize_element(&address.as_hex_string())?;
    }
    seq.end()
}

pub fn deserialize_u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(num) => Ok(num.as_u64().unwrap()),
        serde_json::Value::String(s) => Ok(s.parse::<u64>().unwrap()),
        _ => unimplemented!("unsupported value type"),
    }
}

pub fn serialize_hash<S>(hash: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&const_hex::encode_prefixed(hash))
}

pub fn deserialize_hash<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

    match value {
        serde_json::Value::Array(arr) => {
            let bytes: Vec<u8> = arr
                .into_iter()
                .filter_map(|v| v.as_u64().map(|n| n as u8))
                .collect();
            if bytes.len() != 32 {
                return Err(serde::de::Error::custom(format!(
                    "expected 32 bytes, got {} bytes",
                    bytes.len()
                )));
            }
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes);
            Ok(array)
        }

        serde_json::Value::String(s) => {
            if let Some(stripped) = s.strip_prefix("0x") {
                let decoded = const_hex::decode(stripped).unwrap();
                if decoded.len() != 32 {
                    return Err(serde::de::Error::custom(format!(
                        "expected 32 bytes, got {} bytes",
                        decoded.len()
                    )));
                }
                let mut array = [0u8; 32];
                array.copy_from_slice(&decoded);
                Ok(array)
            } else {
                Err(serde::de::Error::custom(format!(
                    "hex string with 0x prefix",
                )))
            }
        }

        _ => unimplemented!("unsupported value type"),
    }
}

pub fn serialize_merkle_path<S>(paths: &[[u8; 32]], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(paths.len()))?;
    for path in paths {
        seq.serialize_element(&const_hex::encode_prefixed(path))?;
    }
    seq.end()
}

pub fn deserialize_merkle_path<'de, D>(deserializer: D) -> Result<Vec<[u8; 32]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct MerklePathVisitor;

    impl<'de> serde::de::Visitor<'de> for MerklePathVisitor {
        type Value = Vec<[u8; 32]>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a list of 32-byte hex strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut paths = Vec::new();
            while let Some(path) = seq.next_element::<String>()? {
                let decoded = const_hex::decode(&path).map_err(serde::de::Error::custom)?;
                if decoded.len() != 32 {
                    return Err(serde::de::Error::custom(format!(
                        "expected 32 bytes, got {} bytes",
                        decoded.len()
                    )));
                }

                let mut array = [0u8; 32];
                array.copy_from_slice(&decoded);
                paths.push(array);
            }
            Ok(paths)
        }
    }

    deserializer.deserialize_seq(MerklePathVisitor)
}
