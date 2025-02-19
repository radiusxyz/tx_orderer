use std::sync::Arc;

use radius_sdk::{
    json_rpc::client::{Id, RpcClient},
    signature::{Address, ChainType, PrivateKeySigner, Signature},
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

    pub async fn register_tx_orderer(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &str,
        external_rpc_url: &str,
        cluster_rpc_url: &str,
        signer: &PrivateKeySigner,
    ) -> Result<(), SeederError> {
        let message = RegisterTxOrdererMessage {
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
        let parameter = RegisterTxOrderer { message, signature };

        tracing::info!(
            "Register tx_orderer to seeder - address: {:?}, rpc_url: {:?}",
            signer.address().as_hex_string(),
            (external_rpc_url, cluster_rpc_url),
        );

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                RegisterTxOrderer::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(SeederError::Register)
    }

    pub async fn deregister_tx_orderer(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: &str,
        signer: &PrivateKeySigner,
    ) -> Result<(), SeederError> {
        let message = DeregisterTxOrdererMessage {
            platform,
            service_provider,
            cluster_id: cluster_id.to_owned(),
            address: signer.address().to_owned(),
        };
        let signature = signer
            .sign_message(&message)
            .map_err(SeederError::SignMessage)?;
        let parameter = DeregisterTxOrderer { message, signature };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                DeregisterTxOrderer::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(SeederError::Deregister)
    }

    pub async fn get_tx_orderer_rpc_url_list(
        &self,
        tx_orderer_address_list: Vec<String>,
    ) -> Result<GetTxOrdererRpcUrlListResponse, SeederError> {
        let parameter = GetTxOrdererRpcUrlList {
            sequencer_address_list: tx_orderer_address_list,
        };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetTxOrdererRpcUrlList::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(SeederError::GetTxOrdererRpcUrlList)
    }

    pub async fn get_tx_orderer_rpc_url(
        &self,
        tx_orderer_address: String,
    ) -> Result<GetTxOrdererRpcUrlResponse, SeederError> {
        let parameter = GetTxOrdererRpcUrl {
            address: tx_orderer_address,
        };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetTxOrdererRpcUrl::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(SeederError::GetTxOrdererRpcUrl)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterTxOrderer {
    pub message: RegisterTxOrdererMessage,
    pub signature: Signature,
}

impl RegisterTxOrderer {
    pub const METHOD_NAME: &'static str = "register_sequencer";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterTxOrdererMessage {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
    pub address: Address,
    pub external_rpc_url: String,
    pub cluster_rpc_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeregisterTxOrderer {
    pub message: DeregisterTxOrdererMessage,
    pub signature: Signature,
}

impl DeregisterTxOrderer {
    pub const METHOD_NAME: &'static str = "deregister_sequencer";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeregisterTxOrdererMessage {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,

    #[serde(serialize_with = "serialize_address")]
    pub address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTxOrdererRpcUrlList {
    pub sequencer_address_list: Vec<String>,
}

impl GetTxOrdererRpcUrlList {
    pub const METHOD_NAME: &'static str = "get_sequencer_rpc_url_list";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxOrdererRpcInfo {
    #[serde(serialize_with = "serialize_address")]
    pub address: Address,

    pub external_rpc_url: Option<String>,
    pub cluster_rpc_url: Option<String>,
}

impl Default for TxOrdererRpcInfo {
    fn default() -> Self {
        Self {
            address: Address::from_slice(ChainType::Ethereum, &[0u8; 20]).unwrap(),
            external_rpc_url: None,
            cluster_rpc_url: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTxOrdererRpcUrlListResponse {
    pub sequencer_rpc_url_list: Vec<TxOrdererRpcInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTxOrdererRpcUrl {
    address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTxOrdererRpcUrlResponse {
    pub sequencer_rpc_url: TxOrdererRpcInfo,
}

impl GetTxOrdererRpcUrl {
    pub const METHOD_NAME: &'static str = "get_sequencer_rpc_url";
}

#[derive(Debug)]
pub enum SeederError {
    Initialize(radius_sdk::json_rpc::client::RpcClientError),
    Register(radius_sdk::json_rpc::client::RpcClientError),
    Deregister(radius_sdk::json_rpc::client::RpcClientError),
    GetTxOrdererRpcUrlList(radius_sdk::json_rpc::client::RpcClientError),
    GetTxOrdererRpcUrl(radius_sdk::json_rpc::client::RpcClientError),
    SignMessage(radius_sdk::signature::SignatureError),
}

impl std::fmt::Display for SeederError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SeederError {}
