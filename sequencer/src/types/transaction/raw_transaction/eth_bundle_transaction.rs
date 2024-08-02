use crate::types::prelude::*;

// TODO: stompesi
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthRawBundleTransaction(String);

impl From<String> for EthRawBundleTransaction {
    fn from(value: String) -> Self {
        Self(value)
    }
}
