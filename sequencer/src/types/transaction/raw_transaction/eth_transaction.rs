use crate::types::prelude::*;

// TODO: stompesi
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthRawTransaction(String);

impl From<String> for EthRawTransaction {
    fn from(value: String) -> Self {
        Self(value)
    }
}
