use std::fmt::Debug;

use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::rpc::prelude::*;

mod decrypt_transaction;
mod finalize_block;
mod get_encrypted_transaction;
mod get_raw_transaction;
mod send_encrypted_transaction;
mod send_raw_transaction;

pub use decrypt_transaction::*;
pub use finalize_block::*;
pub use get_encrypted_transaction::*;
pub use get_raw_transaction::*;
pub use send_encrypted_transaction::*;
pub use send_raw_transaction::*;

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
