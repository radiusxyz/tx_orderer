use std::{fmt::Debug, time::Duration};

use sequencer_core::{
    caller,
    error::{Error, WrapError},
    serde::de::DeserializeOwned,
};

use crate::parameter::RpcParameter;

pub struct BlockingClient {
    timeout: Duration,
}

impl Default for BlockingClient {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
        }
    }
}

impl BlockingClient {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    pub fn request<T>(&self, parameter: impl RpcParameter) -> Result<T, Error>
    where
        T: Debug + DeserializeOwned,
    {
        let request = RpcRequest::from((parameter, id))
            .to_json_string()
            .wrap_with_context(
                caller!(BlockingClient::request()),
                format_args!("parameter: {:#?}", parameter),
            )?;

        let response = rpc_client
            .http_client
            .post(format!("http://{}", url.as_ref()))
            .header(
                header::CONTENT_TYPE,
                wrap_err!(HeaderValue::from_str("application/json"))?,
            )
            .body(request)
            .send()
            .wrap()?
            .wrap()?;

        let rpc_response: RpcResponse<T> = wrap_err!(serde_json::from_str(&response_string))?;
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
