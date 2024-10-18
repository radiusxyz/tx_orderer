use std::sync::Arc;

use radius_sdk::json_rpc::{Error, RpcClient};
use serde::{Deserialize, Serialize};
use skde::delay_encryption::SecretKey;

pub struct KeyManagementSystemClient {
    inner: Arc<RpcClient>,
}

impl Clone for KeyManagementSystemClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl KeyManagementSystemClient {
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let rpc_client = RpcClient::new(rpc_url)?;

        Ok(Self {
            inner: Arc::new(rpc_client),
        })
    }

    pub async fn get_encryption_key(&self, key_id: u64) -> Result<GetEncryptionKeyReturn, Error> {
        let rpc_parameter = GetEncryptionKey { key_id };

        self.inner
            .request(GetEncryptionKey::METHOD_NAME, rpc_parameter)
            .await
    }

    pub async fn get_decryption_key(&self, key_id: u64) -> Result<GetDecryptionKeyResponse, Error> {
        let rpc_parameter = GetDecryptionKey { key_id };

        self.inner
            .request(GetDecryptionKey::METHOD_NAME, rpc_parameter)
            .await
    }

    pub async fn get_skde_params(&self) -> Result<GetSkdeParamsResponse, Error> {
        let rpc_parameter = GetSkdeParams {};
        self.inner
            .request(GetSkdeParams::METHOD_NAME, rpc_parameter)
            .await
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
