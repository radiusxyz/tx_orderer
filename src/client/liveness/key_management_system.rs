use std::sync::Arc;

use radius_sequencer_sdk::json_rpc::{Error, RpcClient};
use serde::{Deserialize, Serialize};

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

    pub async fn add_key_generator(&self, address: &String, rpc_url: &String) -> Result<(), Error> {
        let message = AddKeyGeneratorMessage {
            address: address.to_owned(),
            rpc_url: rpc_url.to_owned(),
        };
        let parameter = AddKeyGenerator {
            message,
            // signature: vec![].into(),
        };

        self.inner
            .request(AddKeyGenerator::METHOD_NAME, parameter)
            .await
    }

    pub async fn get_key_generator_list(&self) -> Result<GetKeyGeneratorListReturn, Error> {
        let rpc_parameter = GetKeyGeneratorList {};

        self.inner
            .request(GetKeyGeneratorList::METHOD_NAME, rpc_parameter)
            .await
    }

    pub async fn get_encryption_key(&self, key_id: u64) -> Result<GetEncryptionKeyReturn, Error> {
        let rpc_parameter = GetEncryptionKey { key_id };

        self.inner
            .request(GetEncryptionKey::METHOD_NAME, rpc_parameter)
            .await
    }

    pub async fn get_decryption_key(&self, key_id: u64) -> Result<GetDecryptionKeyReturn, Error> {
        let rpc_parameter = GetDecryptionKey { key_id };

        self.inner
            .request(GetDecryptionKey::METHOD_NAME, rpc_parameter)
            .await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddKeyGenerator {
    pub message: AddKeyGeneratorMessage,
    // pub signature: Signature,
}

impl AddKeyGenerator {
    pub const METHOD_NAME: &'static str = "add_key_generator";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddKeyGeneratorMessage {
    pub address: String,
    pub rpc_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetKeyGeneratorList;

impl GetKeyGeneratorList {
    pub const METHOD_NAME: &'static str = "get_key_generator_list";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetKeyGeneratorListReturn {
    pub key_generator_list: Vec<KeyGenerator>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct KeyGenerator {
    address: String,
    ip_address: String,
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
    pub pk: BigUint,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetDecryptionKey {
    pub key_id: u64,
}

impl GetDecryptionKey {
    pub const METHOD_NAME: &'static str = "get_decryption_key";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetDecryptionKeyReturn {
    pub key: SecretKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretKey {
    pub sk: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigUint {
    data: Vec<u64>,
}
