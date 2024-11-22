use std::sync::Arc;

use radius_sdk::{
    json_rpc::client::{Id, RpcClient},
    signature::{Address, PrivateKeySigner, Signature},
};
use serde::{Deserialize, Serialize};

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
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, SeederError> {
        let inner = SeederClientInner {
            rpc_url: rpc_url.as_ref().to_owned(),
            rpc_client: RpcClient::new().map_err(SeederError::Initialize)?,
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
        external_rpc_url: &String,
        cluster_rpc_url: &String,
        signer: &PrivateKeySigner,
    ) -> Result<(), SeederError> {
        let message = RegisterSequencerMessage {
            platform,
            service_provider,
            cluster_id: cluster_id.to_owned(),
            address: signer.address().to_owned(),
            external_rpc_url: external_rpc_url.to_owned(),
            cluster_rpc_url: cluster_rpc_url.to_owned(),
        };
        let signature = signer
            .sign_message(&message)
            .map_err(SeederError::SignMessage)?;
        let parameter = RegisterSequencer { message, signature };

        tracing::info!(
            "Register sequencer to seeder - address: {:?}, rpc_url: {:?}",
            signer.address().as_hex_string(),
            (external_rpc_url, cluster_rpc_url),
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
            .map_err(SeederError::Register)
    }

    pub async fn deregister_sequencer(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &String,
        signer: &PrivateKeySigner,
    ) -> Result<(), SeederError> {
        let message = DeregisterSequencerMessage {
            platform,
            service_provider,
            cluster_id: cluster_id.to_owned(),
            address: signer.address().to_owned(),
        };
        let signature = signer
            .sign_message(&message)
            .map_err(SeederError::SignMessage)?;
        let parameter = DeregisterSequencer { message, signature };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                DeregisterSequencer::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(SeederError::Deregister)
    }

    pub async fn get_sequencer_rpc_url_list(
        &self,
        sequencer_address_list: Vec<String>,
    ) -> Result<GetSequencerRpcUrlListResponse, SeederError> {
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
            .map_err(SeederError::GetSequencerRpcUrlList)
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
    pub external_rpc_url: String,
    pub cluster_rpc_url: String,
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
pub struct SequencerRpcInfo {
    pub address: Address,
    pub external_rpc_url: Option<String>,
    pub cluster_rpc_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlListResponse {
    pub sequencer_rpc_url_list: Vec<SequencerRpcInfo>,
}

#[derive(Debug)]
pub enum SeederError {
    Initialize(radius_sdk::json_rpc::client::RpcClientError),
    Register(radius_sdk::json_rpc::client::RpcClientError),
    Deregister(radius_sdk::json_rpc::client::RpcClientError),
    GetSequencerRpcUrlList(radius_sdk::json_rpc::client::RpcClientError),
    SignMessage(radius_sdk::signature::SignatureError),
}

impl std::fmt::Display for SeederError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SeederError {}
