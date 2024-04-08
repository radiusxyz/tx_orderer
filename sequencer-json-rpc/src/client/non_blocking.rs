use std::{fmt::Debug, time::Duration};

use sequencer_core::{
    caller,
    error::{Error, WrapError},
    reqwest::{
        header::{self, HeaderValue},
        Client, ClientBuilder,
    },
    serde::de::DeserializeOwned,
    serde_json,
};

use crate::{id::Id, parameter::RpcParameter, request::RpcRequest, response::RpcResponse};

pub struct RpcClientBuilder {
    timeout: Duration,
}

impl Default for RpcClientBuilder {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
        }
    }
}

impl RpcClientBuilder {
    pub fn set_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Duration::from_secs(timeout);
        self
    }

    pub fn build(self) -> Result<RpcClient, Error> {
        let http_client = ClientBuilder::default()
            .timeout(self.timeout)
            .build()
            .wrap(caller!(RpcClientBuilder::build()))?;
        Ok(http_client.into())
    }
}

pub struct RpcClient {
    http_client: Client,
}

impl From<Client> for RpcClient {
    fn from(value: Client) -> Self {
        Self { http_client: value }
    }
}

impl RpcClient {
    pub fn builder() -> RpcClientBuilder {
        RpcClientBuilder::default()
    }

    pub async fn request<T>(
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
            .wrap(caller!(RpcClient::request()))?;

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
            .await
            .wrap(caller!(RpcClient::request()))?;

        let response_string = response.text().await.wrap(caller!(RpcClient::request()))?;

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
