use std::fmt::Debug;

use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::rpc::prelude::*;

mod finalize_block;
mod get_encrypted_transaction;
mod send_encrypted_transaction;

pub use finalize_block::*;
pub use get_encrypted_transaction::*;
pub use send_encrypted_transaction::*;

mod get_block;

pub use get_block::*;

#[async_trait]
pub trait RollupRpcParameter: Clone + Debug + DeserializeOwned + Send + Serialize {
    const METHOD_NAME: &'static str;

    type Output: Debug + DeserializeOwned + Send + Serialize;

    fn rpc_method(&self) -> Self {
        self.clone()
    }

    async fn handler(self, context: Arc<AppState>) -> Result<Self::Output, RpcError>;
}
