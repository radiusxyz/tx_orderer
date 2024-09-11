use std::sync::Arc;

use radius_sequencer_sdk::{
    json_rpc::{Error, RpcClient},
    signature::{ChainType, Signature},
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::types::*;

pub struct SeederClient {
    inner: Arc<RpcClient>,
}

impl Clone for SeederClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl SeederClient {
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let rpc_client = RpcClient::new(rpc_url)?;

        Ok(Self {
            inner: Arc::new(rpc_client),
        })
    }

    pub async fn register_sequencer(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &String,
        chain_type: ChainType,
        address: &[u8],
        rpc_url: &String,
    ) -> Result<(), Error> {
        let message = RegisterSequencerMessage {
            platform,
            service_provider,
            cluster_id: cluster_id.to_owned(),
            chain_type,
            address: address.to_vec(),
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
            .request(RegisterSequencer::METHOD_NAME, parameter)
            .await
    }

    pub async fn deregister_sequencer(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &String,
        chain_type: ChainType,
        address: &[u8],
    ) -> Result<(), Error> {
        let message = DeregisterSequencerMessage {
            platform,
            service_provider,
            cluster_id: cluster_id.to_owned(),
            chain_type,
            address: address.to_owned(),
        };
        let parameter = DeregisterSequencer {
            message,
            signature: vec![].into(),
        };

        self.inner
            .request(DeregisterSequencer::METHOD_NAME, parameter)
            .await
    }

    pub async fn get_sequencer_rpc_url_list(
        &self,
        sequencer_address_list: Vec<String>,
    ) -> Result<GetSequencerRpcUrlListResponse, Error> {
        let rpc_parameter = GetSequencerRpcUrlList {
            sequencer_address_list,
        };

        self.inner
            .request(GetSequencerRpcUrlList::METHOD_NAME, rpc_parameter)
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
    pub chain_type: ChainType,
    pub address: Vec<u8>,
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
    pub chain_type: ChainType,
    pub address: Vec<u8>,
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
