mod encrypted_transaction;
mod raw_transaction;

pub use encrypted_transaction::*;
pub use raw_transaction::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptedTransactionType {
    Pvde,
    Skde,
    NotSupport,
}

impl Default for EncryptedTransactionType {
    fn default() -> Self {
        Self::NotSupport
    }
}

impl From<String> for EncryptedTransactionType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "pvde" | "Pvde" | "PVDE" => Self::Pvde,
            "skde" | "Skde" | "SKDE" => Self::Skde,
            _ => Self::NotSupport,
        }
    }
}
