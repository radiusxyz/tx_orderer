use std::sync::Arc;

use radius_sdk::{
    json_rpc::client::{Id, RpcClient, RpcClientError},
    signature::{Address, Signature},
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::types::*;

pub struct SeederClient {
    inner: Arc<SeederClientInner>,
}

struct SeederClientInner {
    rpc_url: String,
    rpc_client: RpcClient,
}

impl Clone for SeederClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl SeederClient {
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, RpcClientError> {
        let inner = SeederClientInner {
            rpc_url: rpc_url.as_ref().to_owned(),
            rpc_client: RpcClient::new()?,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub async fn register_sequencer(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &String,
        address: &Address,
        rpc_url: &String,
    ) -> Result<(), RpcClientError> {
        let message = RegisterSequencerMessage {
            platform,
            service_provider,
            cluster_id: cluster_id.to_owned(),
            address: address.clone(),
            rpc_url: rpc_url.to_owned(),
        };
        let parameter = RegisterSequencer {
            message,
            signature: vec![].into(),
        };

        info!(
            "Register sequencer to seeder - address: {:?}, rpc_url: {:?}",
            address, rpc_url
        );

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                RegisterSequencer::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
    }

    pub async fn deregister_sequencer(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &String,
        address: &Address,
    ) -> Result<(), RpcClientError> {
        let message = DeregisterSequencerMessage {
            platform,
            service_provider,
            cluster_id: cluster_id.to_owned(),
            address: address.clone(),
        };
        let parameter = DeregisterSequencer {
            message,
            signature: vec![].into(),
        };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                DeregisterSequencer::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
    }

    pub async fn get_sequencer_rpc_url_list(
        &self,
        sequencer_address_list: Vec<String>,
    ) -> Result<GetSequencerRpcUrlListResponse, RpcClientError> {
        let parameter = GetSequencerRpcUrlList {
            sequencer_address_list,
        };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetSequencerRpcUrlList::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSequencer {
    pub message: RegisterSequencerMessage,
    pub signature: Signature,
}

impl RegisterSequencer {
    pub const METHOD_NAME: &'static str = "register_sequencer";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSequencerMessage {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
    pub address: Address,
    pub rpc_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeregisterSequencer {
    pub message: DeregisterSequencerMessage,
    pub signature: Signature,
}

impl DeregisterSequencer {
    pub const METHOD_NAME: &'static str = "deregister_sequencer";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeregisterSequencerMessage {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
    pub address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlList {
    pub sequencer_address_list: Vec<String>,
}

impl GetSequencerRpcUrlList {
    pub const METHOD_NAME: &'static str = "get_sequencer_rpc_url_list";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlListResponse {
    pub sequencer_rpc_url_list: Vec<(String, Option<String>)>,
}
