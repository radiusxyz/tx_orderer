use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::json_rpc::{Error as JsonRpcError, ErrorKind, RpcClient};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::{
    error::Error,
    rpc::cluster::{SyncBlock, SyncTransaction},
    types::*,
};

pub struct SequencerClient(Arc<RpcClient>);

unsafe impl Send for SequencerClient {}

unsafe impl Sync for SequencerClient {}

impl Clone for SequencerClient {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RegisterRpcUrlResponse {
    pub success: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GetRpcUrlsResponse {
    pub rpc_urls: HashMap<Address, IpAddress>,
}

impl SequencerClient {
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let client = RpcClient::new(rpc_url).map_err(|error| {
            Error::RpcError(JsonRpcError::custom(ErrorKind::BuildClient, error))
        })?;

        Ok(Self(Arc::new(client)))
    }

    pub async fn register_rpc_url(
        &self,
        address: Address,
        rpc_url: IpAddress,
    ) -> Result<(), Error> {
        let rpc_method = json!({
            "address": address,
            "rpc_url": rpc_url,
        });

        info!("Get register_rpc_url - rpc_method: {:?}", rpc_method);

        let register_rpc_url_response: RegisterRpcUrlResponse =
            self.0.request("register_rpc_url", rpc_method).await?;

        if !register_rpc_url_response.success {
            return Err(Error::RegisterRpcUrl);
        }

        Ok(())
    }

    pub async fn sync_transaction(&self, parameter: SyncTransaction) -> Result<(), Error> {
        self.0
            .request::<SyncTransaction, ()>(SyncTransaction::METHOD_NAME, parameter)
            .await?;

        Ok(())
    }

    pub async fn sync_block(&self, parameter: SyncBlock) -> Result<(), Error> {
        self.0
            .request::<SyncBlock, ()>(SyncBlock::METHOD_NAME, parameter)
            .await?;

        Ok(())
    }
}
