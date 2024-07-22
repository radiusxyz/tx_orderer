use async_trait::async_trait;

use crate::{
    impl_rollup_rpc_forwarder,
    rpc::{
        external::{forward_to_rollup_rpc_request, RollupRpcParameter},
        prelude::*,
    },
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthGetBlockByNumber {
    block_number: String,
    full_tx: bool,
}

impl_rollup_rpc_forwarder!(EthGetBlockByNumber, "eth_getBlockByNumber", String);
