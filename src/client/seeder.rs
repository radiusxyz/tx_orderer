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
    rpc_client: Arc<RpcClient>,
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
        liveness_service_provider: LivenessServiceProvider,
        cluster_id: &ClusterId,
        external_rpc_url: &str,
        cluster_rpc_url: &str,
        signer: &PrivateKeySigner,
    ) -> Result<(), SeederError> {
        let message = RegisterTxOrdererMessage {
            platform,
            liveness_service_provider,
            cluster_id: cluster_id.to_owned(),
            tx_orderer_address: signer.address().to_owned(),
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
        liveness_service_provider: LivenessServiceProvider,
        cluster_id: &ClusterId,
        signer: &PrivateKeySigner,
    ) -> Result<(), SeederError> {
        let message = DeregisterTxOrdererMessage {
            platform,
            liveness_service_provider,
            cluster_id: cluster_id.to_owned(),
            tx_orderer_address: signer.address().to_owned(),
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

    pub async fn get_tx_orderer_rpc_info_list(
        &self,
        tx_orderer_address_list: Vec<String>,
    ) -> Result<GetTxOrdererRpcInfoListResponse, SeederError> {
        let parameter = GetTxOrdererRpcInfoList {
            tx_orderer_address_list,
        };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetTxOrdererRpcInfoList::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(SeederError::GetTxOrdererInfoList)
    }

    pub async fn get_tx_orderer_rpc_info(
        &self,
        tx_orderer_address: String,
    ) -> Result<GetTxOrdererRpcInfoResponse, SeederError> {
        let parameter = GetTxOrdererRpcInfo { tx_orderer_address };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetTxOrdererRpcInfo::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(SeederError::GetTxOrdererInfo)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterTxOrderer {
    pub message: RegisterTxOrdererMessage,
    pub signature: Signature,
}

impl RegisterTxOrderer {
    pub const METHOD_NAME: &'static str = "register_tx_orderer";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterTxOrdererMessage {
    pub platform: Platform,
    pub liveness_service_provider: LivenessServiceProvider,
    pub cluster_id: ClusterId,
    pub tx_orderer_address: Address,
    pub external_rpc_url: String,
    pub cluster_rpc_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeregisterTxOrderer {
    pub message: DeregisterTxOrdererMessage,
    pub signature: Signature,
}

impl DeregisterTxOrderer {
    pub const METHOD_NAME: &'static str = "deregister_tx_orderer";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeregisterTxOrdererMessage {
    pub platform: Platform,
    pub liveness_service_provider: LivenessServiceProvider,
    pub cluster_id: ClusterId,

    #[serde(serialize_with = "serialize_address")]
    pub tx_orderer_address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTxOrdererRpcInfoList {
    pub tx_orderer_address_list: Vec<String>,
}

impl GetTxOrdererRpcInfoList {
    pub const METHOD_NAME: &'static str = "get_tx_orderer_rpc_info_list";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxOrdererRpcInfo {
    #[serde(serialize_with = "serialize_address")]
    pub tx_orderer_address: Address,

    pub external_rpc_url: Option<String>,
    pub cluster_rpc_url: Option<String>,
}

impl Default for TxOrdererRpcInfo {
    fn default() -> Self {
        Self {
            tx_orderer_address: Address::from_slice(ChainType::Ethereum, &[0u8; 20]).unwrap(),
            external_rpc_url: None,
            cluster_rpc_url: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTxOrdererRpcInfoListResponse {
    pub tx_orderer_rpc_info_list: Vec<TxOrdererRpcInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTxOrdererRpcInfo {
    tx_orderer_address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTxOrdererRpcInfoResponse {
    pub tx_orderer_rpc_info: TxOrdererRpcInfo,
}

impl GetTxOrdererRpcInfo {
    pub const METHOD_NAME: &'static str = "get_tx_orderer_rpc_info";
}

#[derive(Debug)]
pub enum SeederError {
    Initialize(radius_sdk::json_rpc::client::RpcClientError),
    Register(radius_sdk::json_rpc::client::RpcClientError),
    Deregister(radius_sdk::json_rpc::client::RpcClientError),
    GetTxOrdererInfoList(radius_sdk::json_rpc::client::RpcClientError),
    GetTxOrdererInfo(radius_sdk::json_rpc::client::RpcClientError),
    SignMessage(radius_sdk::signature::SignatureError),
}

impl std::fmt::Display for SeederError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SeederError {}
