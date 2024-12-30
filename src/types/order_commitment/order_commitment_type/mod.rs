mod sign_order_commitment;
mod transaction_hash_order_commitment;

use std::str::FromStr;

use serde::{Deserialize, Serialize};
pub use sign_order_commitment::*;
pub use transaction_hash_order_commitment::*;

use crate::error::Error;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderCommitmentType {
    TransactionHash,
    Sign,
}

impl FromStr for OrderCommitmentType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "transaction_hash" | "TransactionHash" => Ok(Self::TransactionHash),
            "sign" | "Sign" => Ok(Self::Sign),
            _ => Err(Error::UnsupportedOrderCommitmentType),
        }
    }
}
