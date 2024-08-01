use crate::types::prelude::*;

// TODO: stompesi
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthBundleRawTransaction(String);

impl From<String> for EthBundleRawTransaction {
    fn from(value: String) -> Self {
        Self(value)
    }
}
