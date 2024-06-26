use std::pin::Pin;

use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
};
use serde::{de::DeserializeOwned, ser::Serialize};
use tokio::time::{sleep, Duration};

use crate::{types::Parameter, Error, ErrorKind};

pub struct RpcClient {
    http_client: HttpClient,
    timeout: u64,
    retry: u8,
    retry_interval: u64,
}

impl RpcClient {
    pub const DEFAULT_TIMEOUT: u64 = 3;
    pub const DEFAULT_RETRY: u8 = 0;
    pub const DEFAULT_RETRY_INTERVAL: u64 = 0;

    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let http_client = HttpClientBuilder::new()
            .request_timeout(Duration::from_secs(Self::DEFAULT_TIMEOUT))
            .build(rpc_url.as_ref())
            .map_err(|error| (ErrorKind::BuildClient, error))?;

        Ok(Self {
            http_client,
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

    async fn request_inner<P, R>(&self, method: &'static str, parameter: P) -> Result<R, Error>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        let parameter = Parameter::from(parameter);
        self.http_client
            .request(method, parameter)
            .await
            .map_err(|error| (ErrorKind::RpcRequest, error).into())
    }

    pub async fn request<P, R>(&self, method: &'static str, parameter: P) -> Result<R, Error>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        if self.retry != 0 {
            for _ in 0..self.retry {
                if let Some(response) = self.request_inner(method, parameter.clone()).await.ok() {
                    return Ok(response);
                } else {
                    sleep(Duration::from_secs(self.retry_interval)).await;
                }
            }
        }

        self.request_inner(method, parameter).await
    }

    pub async fn fetch<P, R>(
        rpc_addresses: Vec<impl AsRef<str>>,
        method: &'static str,
        parameter: P,
    ) -> Result<R, Error>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        let rpc_client_list: Vec<RpcClient> = rpc_addresses
            .into_iter()
            .filter_map(|rpc_address| RpcClient::new(rpc_address.as_ref()).ok())
            .collect();
        let fused_futures: Vec<Pin<Box<Fuse<_>>>> = rpc_client_list
            .iter()
            .map(|client| Box::pin(client.request::<P, R>(method, parameter.clone()).fuse()))
            .collect();

        let (rpc_response, _): (R, Vec<_>) = select_ok(fused_futures)
            .await
            .map_err(|_| Error::custom(ErrorKind::Fetch, "None of the requests returned `Ok`"))?;

        Ok(rpc_response)
    }
}
