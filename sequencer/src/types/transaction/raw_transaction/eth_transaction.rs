use crate::types::prelude::*;

// TODO: stompesi
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthTransaction {
    raw_transaction: String,
}
