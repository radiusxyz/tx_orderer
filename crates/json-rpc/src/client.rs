use std::time::Duration;

use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
};
use serde::{de::DeserializeOwned, ser::Serialize};

use crate::{types::Parameter, Error, ErrorKind};

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

    pub async fn request<P, R>(&self, method: &'static str, parameter: P) -> Result<R, Error>
    where
        P: Serialize + Send,
        R: DeserializeOwned,
    {
        let parameter = Parameter::from(parameter);
        self.http_client
            .request(method, parameter)
            .await
            .map_err(|error| (ErrorKind::RpcRequest, error).into())
    }
}
