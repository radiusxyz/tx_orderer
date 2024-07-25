use crate::types::prelude::*;

// TODO: stompesi
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthBundleTransaction {
    raw_transaction: String,
}
