use std::sync::Arc;

use radius_sequencer_sdk::{
    json_rpc::{Error, RpcClient},
    signature::{ChainType, Signature},
};
use serde::{Deserialize, Serialize};

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

    // pub async fn register(
    //     &self,
    //     platform: Platform,
    //     service_provider: ServiceProvider,
    //     cluster_id: String,
    //     chain_type: ChainType,
    //     address: Vec<u8>,
    //     rpc_url: String,
    // ) -> Result<(), Error> {
    //     let rpc_parameter = RegisterSequencer {
    //         message: RegisterSequencerMessage {
    //             platform,
    //             service_provider,
    //             cluster_id,
    //             chain_type,
    //             address,
    //             rpc_url,
    //         },
    //         signature: vec![],
    //     };
    //     Ok(())
    // }

    // pub async fn deregister(
    //     &self,
    //     platform: Platform,
    //     service_provider: ServiceProvider,
    //     cluster_id: String,
    //     chain_type: ChainType,
    //     address: Vec<u8>,
    // ) -> Result<(), Error> {
    //     let rpc_parameter = DeregisterSequencer {
    //         message: DeregisterSequencerMessage {
    //             platform,
    //             service_provider,
    //             cluster_id,
    //             chain_type,
    //             address,
    //         }
    //         signature: vec![],
    //     };

    //     Ok(())
    // }

    pub async fn get_cluster_info(
        &self,
        sequencer_address_list: Vec<String>,
        rollup_address_list: Vec<String>,
    ) -> Result<GetClusterInfoReturn, Error> {
        let rpc_parameter = GetClusterInfo {
            sequencer_address_list,
            rollup_address_list,
        };

        self.inner
            .request(GetClusterInfo::METHOD_NAME, rpc_parameter)
            .await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSequencer {
    message: RegisterSequencerMessage,
    signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSequencerMessage {
    platform: Platform,
    service_provider: ServiceProvider,
    cluster_id: String,
    chain_type: ChainType,
    address: Vec<u8>,
    rpc_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeregisterSequencer {
    message: DeregisterSequencerMessage,
    signature: Signature,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeregisterSequencerMessage {
    platform: Platform,
    service_provider: ServiceProvider,
    cluster_id: String,
    chain_type: ChainType,
    address: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterInfo {
    sequencer_address_list: Vec<String>,
    rollup_address_list: Vec<String>,
}

impl GetClusterInfo {
    pub const METHOD_NAME: &'static str = "get_cluster_info";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterInfoReturn {
    pub sequencer_info: Vec<(String, Option<String>)>,
    pub cluster_info: Vec<(String, Option<String>)>,
}
