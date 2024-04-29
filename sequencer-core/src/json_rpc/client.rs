use std::time::Duration;

use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
};

use crate::{
    context,
    error::{Error, WrapError},
    json_rpc::method::{RpcMethod, RpcParam},
};

pub struct RpcClient {
    http_client: HttpClient,
}

impl RpcClient {
    pub fn new(endpoint: impl AsRef<str>, timeout: u64) -> Result<Self, Error> {
        let endpoint = endpoint.as_ref();
        let http_client = HttpClientBuilder::new()
            .request_timeout(Duration::from_secs(timeout))
            .build(endpoint)
            .wrap(context!(endpoint))?;
        Ok(Self { http_client })
    }

    pub async fn request<T>(&self, method: T) -> Result<T::Response, Error>
    where
        T: RpcMethod + Into<RpcParam<T>>,
    {
        let method_name = T::method_name();
        let rpc_parameter = RpcParam::from(method);
        let rpc_response = self
            .http_client
            .request(method_name, rpc_parameter)
            .await
            .wrap(context!(method_name))?;
        Ok(rpc_response)
    }
}
