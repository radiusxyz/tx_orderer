use async_trait::async_trait;

use crate::{
    impl_rollup_rpc_forwarder,
    rpc::{
        external::{forward_to_rollup_rpc_request, RollupRpcParameter},
        prelude::*,
    },
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthTransactionForEstimateGas {
    pub from: String,
    pub to: String,
    #[serde(default, rename = "gasPrice")]
    pub gas_price: String,
    pub value: String,
    pub data: String,
}

impl_rollup_rpc_forwarder!(EthTransactionForEstimateGas, "eth_estimateGas", String);
