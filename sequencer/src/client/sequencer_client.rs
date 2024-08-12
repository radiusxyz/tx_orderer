use std::{collections::HashMap, pin::Pin, sync::Arc};

use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use radius_sequencer_sdk::json_rpc::{Error as JsonRpcError, ErrorKind, RpcClient};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::{
    error::Error,
    rpc::cluster::{SyncBlock, SyncPartialKey, SyncTransaction},
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
        self.request::<SyncTransaction, ()>(SyncTransaction::METHOD_NAME, parameter)
            .await?;

        Ok(())
    }

    pub async fn sync_block(&self, parameter: SyncBlock) -> Result<(), Error> {
        self.request::<SyncBlock, ()>(SyncBlock::METHOD_NAME, parameter)
            .await?;

        Ok(())
    }

    pub async fn request<P, R>(&self, method: &'static str, params: P) -> Result<R, Error>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        let result = self.0.request::<P, R>(method, params).await?;

        Ok(result)
    }

    async fn fetch<P, R>(
        sequencer_rpc_client_list: &Vec<Self>,
        method: &'static str,
        params: P,
    ) -> Result<R, Error>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        let fused_futures: Vec<Pin<Box<Fuse<_>>>> = sequencer_rpc_client_list
            .iter()
            .map(|sequencer_rpc_client| {
                Box::pin(
                    sequencer_rpc_client
                        .request::<P, R>(method, params.clone())
                        .fuse(),
                )
            })
            .collect();

        let (rpc_response, _): (R, Vec<_>) = select_ok(fused_futures)
            .await
            .map_err(|_| Error::FetchResponse)?;

        Ok(rpc_response)
    }
}

impl SequencerClient {
    pub async fn sync_partial_key(&self, parameter: SyncPartialKey) -> Result<(), Error> {
        self.request::<SyncPartialKey, ()>(SyncPartialKey::METHOD_NAME, parameter)
            .await?;

        Ok(())
    }
}
