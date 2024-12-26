mod block;
mod cluster;
mod config;
mod merkle;
mod order_commitment;
mod rollup;
mod sequencing;
mod time_lock_puzzle;
mod transaction;
mod validation;
mod zkp;

pub use block::*;
pub use cluster::*;
pub use config::*;
pub use merkle::*;
pub use order_commitment::*;
use radius_sdk::signature::Address;
pub use rollup::*;
pub use sequencing::*;
use serde::ser::SerializeSeq;
pub use time_lock_puzzle::*;
pub use transaction::*;
pub use validation::*;
pub use zkp::*;

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

fn serialize_address_list<S>(addresses: &Vec<Address>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(addresses.len()))?;
    for address in addresses {
        seq.serialize_element(&address.as_hex_string())?;
    }
    seq.end()
}

fn serialize_merkle_path<S>(paths: &Vec<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(paths.len()))?;
    for path in paths {
        seq.serialize_element(&const_hex::encode_prefixed(path))?;
    }
    seq.end()
}

fn deserialize_merkle_path<'de, D>(deserializer: D) -> Result<Vec<[u8; 32]>, D::Error>
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
