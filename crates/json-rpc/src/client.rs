use std::time::Duration;

use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
};

use crate::{
    method::{RpcMethod, RpcParam},
    Error, ErrorKind,
};

pub struct RpcClient {
    http_client: HttpClient,
}

impl RpcClient {
    pub fn new(rpc_address: impl AsRef<str>, timeout: u64) -> Result<Self, Error> {
        let endpoint = format!("http://{}", rpc_address.as_ref());
        let http_client = HttpClientBuilder::new()
            .request_timeout(Duration::from_secs(timeout))
            .build(endpoint)
            .map_err(|error| (ErrorKind::BuildClient, error))?;
        Ok(Self { http_client })
    }

    pub async fn request<T>(&self, method: T) -> Result<<T as RpcMethod>::Response, Error>
    where
        T: RpcMethod + Into<RpcParam<T>> + Send,
    {
        self.http_client
            .request::<<T as RpcMethod>::Response, RpcParam<T>>(T::method_name(), method.into())
            .await
            .map_err(|error| (ErrorKind::RpcRequest, error).into())
    }
}
