use async_trait::async_trait;

use crate::{
    impl_rollup_rpc_forwarder,
    rpc::{
        external::{forward_to_rollup_rpc_request, RollupRpcParameter},
        prelude::*,
    },
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthCall {
    tx_data: EthTxData,
    _something: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EthTxData {
    to: String,
    data: String,
    from: Option<String>,
}

impl_rollup_rpc_forwarder!(EthCall, "eth_call", String);
