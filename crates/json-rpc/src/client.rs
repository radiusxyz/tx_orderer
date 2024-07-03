use std::sync::Arc;

use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
};
use serde::{de::DeserializeOwned, ser::Serialize};
use tokio::time::{sleep, Duration};

use crate::{types::Parameter, Error, ErrorKind};

pub struct RpcClient {
    http_client: Arc<HttpClient>,
    timeout: u64,
    retry: u8,
    retry_interval: u64,
}

unsafe impl Send for RpcClient {}

unsafe impl Sync for RpcClient {}

impl Clone for RpcClient {
    fn clone(&self) -> Self {
        Self {
            http_client: self.http_client.clone(),
            timeout: self.timeout,
            retry: self.retry,
            retry_interval: self.retry_interval,
        }
    }
}

impl RpcClient {
    pub const DEFAULT_TIMEOUT: u64 = 5;
    pub const DEFAULT_RETRY: u8 = 0;
    pub const DEFAULT_RETRY_INTERVAL: u64 = 0;

    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let http_client = HttpClientBuilder::new()
            .request_timeout(Duration::from_secs(Self::DEFAULT_TIMEOUT))
            .build(rpc_url.as_ref())
            .map_err(|error| (ErrorKind::BuildClient, error))?;

        Ok(Self {
            http_client: Arc::new(http_client),
            timeout: Self::DEFAULT_TIMEOUT,
            retry: Self::DEFAULT_RETRY,
            retry_interval: Self::DEFAULT_RETRY_INTERVAL,
        })
    }

    pub fn timeout(mut self, value: u64) -> Self {
        self.timeout = value;
        self
    }

    pub fn max_retry(mut self, value: u8) -> Self {
        self.retry = value;
        self
    }

    /// Retry interval in seconds
    pub fn retry_interval(mut self, value: u64) -> Self {
        self.retry_interval = value;
        self
    }

    async fn request_inner<P, R>(&self, name: &'static str, method: P) -> Result<R, Error>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        let method = Parameter::from(method);
        self.http_client
            .request(name, method)
            .await
            .map_err(|error| (ErrorKind::RpcRequest, error).into())
    }

    pub async fn request<P, R>(&self, name: &'static str, method: P) -> Result<R, Error>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        if self.retry != 0 {
            for _ in 0..self.retry {
                if let Some(response) = self.request_inner(name, method.clone()).await.ok() {
                    return Ok(response);
                } else {
                    sleep(Duration::from_secs(self.retry_interval)).await;
                }
            }
        }

        self.request_inner(name, method).await
    }
}
