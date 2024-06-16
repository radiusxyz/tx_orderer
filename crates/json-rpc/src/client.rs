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
    retry: u8,
    retry_interval: u64,
}

impl RpcClient {
    pub fn new(rpc_address: impl AsRef<str>, timeout: u64) -> Result<Self, Error> {
        let endpoint = format!("http://{}", rpc_address.as_ref());
        let http_client = HttpClientBuilder::new()
            .request_timeout(Duration::from_secs(timeout))
            .build(endpoint)
            .map_err(|error| (ErrorKind::BuildClient, error))?;
        Ok(Self {
            http_client,
            retry: 0,
            retry_interval: 0,
        })
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
        timeout: u64,
        method: &'static str,
        parameter: P,
    ) -> Result<R, Error>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        let rpc_client_list: Vec<RpcClient> = rpc_addresses
            .iter()
            .filter_map(|rpc_address| RpcClient::new(rpc_address.as_ref(), timeout).ok())
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
