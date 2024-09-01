use radius_sequencer_sdk::json_rpc::RpcClient;
use serde::{Deserialize, Serialize};

use crate::error::Error;

pub struct SeederClient(RpcClient);

impl SeederClient {
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let rpc_client = RpcClient::new(rpc_url).unwrap();

        Ok(Self(rpc_client))
    }

    pub fn register(&self) {}

    pub fn deregister(&self) {}

    pub fn get_rpc_url_list(&self) {}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRpcUrlList {}
