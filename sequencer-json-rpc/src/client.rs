use std::time::Duration;

use sequencer_core::{
    caller,
    error::{Error, WrapError},
    jsonrpsee::{
        core::client::ClientT,
        http_client::{HttpClient, HttpClientBuilder},
    },
};

use crate::method::RpcMethod;

pub struct RpcClient {
    http_client: HttpClient,
}

impl RpcClient {
    pub fn new(endpoint: impl AsRef<str>, timeout: u64) -> Result<Self, Error> {
        let http_client = HttpClientBuilder::new()
            .request_timeout(Duration::from_secs(timeout))
            .build(endpoint.as_ref())
            .wrap(caller!(RpcClient::new()))?;
        Ok(Self { http_client })
    }

    pub async fn request<T>(&self, method: T) -> Result<T::Response, Error>
    where
        T: RpcMethod,
    {
        let rpc_response = self
            .http_client
            .request(T::method_name(), method)
            .await
            .wrap(caller!(RpcClient::request()))?;
        Ok(rpc_response)
    }
}
