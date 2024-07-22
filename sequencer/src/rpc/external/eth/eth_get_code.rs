use async_trait::async_trait;

use crate::{
    impl_rollup_rpc_forwarder,
    rpc::{
        external::{forward_to_rollup_rpc_request, RollupRpcParameter},
        prelude::*,
    },
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthGetCode {
    address: String,
    block_number: String,
}

impl_rollup_rpc_forwarder!(EthGetCode, "eth_getCode", String);
