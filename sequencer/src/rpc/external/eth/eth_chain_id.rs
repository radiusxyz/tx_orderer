use async_trait::async_trait;

use crate::{
    impl_rollup_rpc_forwarder,
    rpc::{
        external::{forward_to_rollup_rpc_request, RollupRpcParameter},
        prelude::*,
    },
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthChainId {}

#[async_trait]
impl RollupRpcParameter for EthChainId {
    const METHOD_NAME: &'static str = stringify!(eth_chainId);

    type Output = String;

    fn rpc_method(&self) -> Self {
        self.clone()
    }

    async fn handler(self, context: Arc<AppState>) -> Result<Self::Output, RpcError> {
        let parameter = self.rpc_method();
        let eth_rpc_endpoint = context.config().ethereum_rpc_url().clone();

        Ok(forward_to_rollup_rpc_request(parameter, eth_rpc_endpoint).await?)
    }
}

impl_rollup_rpc_forwarder!(EthChainId, "eth_chainId", String);
