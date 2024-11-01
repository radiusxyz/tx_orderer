use std::sync::Arc;

use radius_sdk::json_rpc::client::{Id, RpcClient};
use serde::{Deserialize, Serialize};
use skde::delay_encryption::SecretKey;

pub struct KeyManagementSystemClient {
    inner: Arc<KeyManagementSystemClientInner>,
}

struct KeyManagementSystemClientInner {
    rpc_url: String,
    rpc_client: RpcClient,
}

impl Clone for KeyManagementSystemClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl KeyManagementSystemClient {
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, KeyManagementSystemError> {
        let inner = KeyManagementSystemClientInner {
            rpc_url: rpc_url.as_ref().to_owned(),
            rpc_client: RpcClient::new().map_err(KeyManagementSystemError::Initialize)?,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub async fn get_encryption_key(
        &self,
        key_id: u64,
    ) -> Result<GetEncryptionKeyReturn, KeyManagementSystemError> {
        let parameter = GetEncryptionKey { key_id };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetEncryptionKey::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(KeyManagementSystemError::GetEncryptionKey)
    }

    pub async fn get_decryption_key(
        &self,
        key_id: u64,
    ) -> Result<GetDecryptionKeyResponse, KeyManagementSystemError> {
        let parameter = GetDecryptionKey { key_id };

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetDecryptionKey::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(KeyManagementSystemError::GetDecryptionKey)
    }

    pub async fn get_skde_params(&self) -> Result<GetSkdeParamsResponse, KeyManagementSystemError> {
        let parameter = GetSkdeParams {};

        self.inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetSkdeParams::METHOD_NAME,
                &parameter,
                Id::Null,
            )
            .await
            .map_err(KeyManagementSystemError::GetSkdeParams)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptionKey {
    pub key_id: u64,
}

impl GetEncryptionKey {
    pub const METHOD_NAME: &'static str = "get_encryption_key";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptionKeyReturn {
    pub key: PublicKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub pk: skde::BigUint,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetDecryptionKey {
    pub key_id: u64,
}

impl GetDecryptionKey {
    pub const METHOD_NAME: &'static str = "get_decryption_key";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetDecryptionKeyResponse {
    pub decryption_key: SecretKey,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSkdeParams {}

impl GetSkdeParams {
    pub const METHOD_NAME: &'static str = "get_skde_params";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSkdeParamsResponse {
    pub skde_params: skde::delay_encryption::SkdeParams,
}

#[derive(Debug)]
pub enum KeyManagementSystemError {
    Initialize(radius_sdk::json_rpc::client::RpcClientError),
    GetEncryptionKey(radius_sdk::json_rpc::client::RpcClientError),
    GetDecryptionKey(radius_sdk::json_rpc::client::RpcClientError),
    GetSkdeParams(radius_sdk::json_rpc::client::RpcClientError),
}

impl std::fmt::Display for KeyManagementSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for KeyManagementSystemError {}
