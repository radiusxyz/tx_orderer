use std::{fmt::Debug, time::Duration};

use sequencer_core::{
    caller,
    error::{Error, WrapError},
    reqwest::{
        blocking::{Client, ClientBuilder},
        header::{self, HeaderValue},
    },
    serde::de::DeserializeOwned,
    serde_json,
};

use crate::{id::Id, parameter::RpcParameter, request::RpcRequest, response::RpcResponse};

pub struct BlockingRpcClientBuilder {
    timeout: Duration,
}

impl Default for BlockingRpcClientBuilder {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
        }
    }
}

impl BlockingRpcClientBuilder {
    pub fn set_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Duration::from_secs(timeout);
        self
    }

    pub fn build(self) -> Result<BlockingRpcClient, Error> {
        let http_client = ClientBuilder::default()
            .timeout(self.timeout)
            .build()
            .wrap(caller!(BlockingRpcClientBuilder::build()))?;
        Ok(http_client.into())
    }
}

pub struct BlockingRpcClient {
    http_client: Client,
}

impl From<Client> for BlockingRpcClient {
    fn from(value: Client) -> Self {
        Self { http_client: value }
    }
}

impl BlockingRpcClient {
    pub fn builder() -> BlockingRpcClientBuilder {
        BlockingRpcClientBuilder::default()
    }

    pub fn request<T>(
        &self,
        url: impl AsRef<str>,
        parameter: impl RpcParameter,
        id: Id,
    ) -> Result<T, Error>
    where
        T: Debug + DeserializeOwned,
    {
        let request = RpcRequest::from((parameter, id))
            .to_json_string()
            .wrap(caller!(BlockingRpcClient::request()))?;

        let response = self
            .http_client
            .post(format!("http://{}", url.as_ref()))
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str("application/json").wrap_context(
                    caller!(BlockingRpcClient::request()),
                    "header value: 'application/json'",
                )?,
            )
            .body(request)
            .send()
            .wrap(caller!(BlockingRpcClient::request()))?;

        let response_string = response
            .text()
            .wrap(caller!(BlockingRpcClient::request()))?;

        let rpc_response: RpcResponse<T> = serde_json::from_str(&response_string).wrap_context(
            caller!(BlockingRpcClient::request()),
            format_args!("response String: {:?}", response_string),
        )?;
        match rpc_response {
            RpcResponse::Result {
                jsonrpc: _,
                result,
                id: _,
            } => Ok(result),
            RpcResponse::Error {
                jsonrpc: _,
                error,
                id: _,
            } => Err(Error::new(caller!(BlockingClient::request()), error)),
        }
    }
}
