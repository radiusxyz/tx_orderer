mod eth;
mod send_transaction;

use std::fmt::Debug;

use async_trait::async_trait;
pub use send_transaction::SendTransaction;
use serde::de::DeserializeOwned;

use crate::rpc::prelude::*;

#[async_trait]
pub trait RollupRpcParameter: Clone + Debug + DeserializeOwned + Send + Serialize {
    const METHOD_NAME: &'static str;

    type Output: Debug + DeserializeOwned + Send + Serialize;

    fn rpc_method(&self) -> Self {
        self.clone()
    }

    async fn handler(self, context: Arc<AppState>) -> Result<Self::Output, RpcError>;
}

pub async fn forward_to_rollup_rpc_request<T: RollupRpcParameter>(
    rpc_parameter: T,
    rollup_rpc_url: String,
) -> Result<T::Output, RpcError> {
    let rpc_client = RpcClient::new(rollup_rpc_url)?;

    rpc_client
        .request(T::METHOD_NAME, rpc_parameter.rpc_method())
        .await
        .map_err(|error| error.into())
}

#[macro_export]
macro_rules! impl_rollup_rpc_forwarder {
    ($method_struct:ident, $method_name:expr, $output_type:ty) => {
        #[async_trait]
        impl RollupRpcParameter for $method_struct {
            const METHOD_NAME: &'static str = stringify!($method_name);

            type Output = $output_type;

            fn rpc_method(&self) -> Self {
                self.clone()
            }

            async fn handler(self, context: Arc<AppState>) -> Result<Self::Output, RpcError> {
                let parameter = self.rpc_method();
                let eth_rpc_url = context.config().ethereum_rpc_url().clone();

                Ok(forward_to_rollup_rpc_request(parameter, eth_rpc_url).await?)
            }
        }
    };
}
