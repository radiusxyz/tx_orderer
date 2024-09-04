use std::sync::Arc;

use radius_sequencer_sdk::json_rpc::{Error, RpcClient};

pub struct SeederClient {
    inner: Arc<RpcClient>,
}

impl SeederClient {
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let rpc_client = RpcClient::new(rpc_url)?;

        Ok(Self {
            inner: Arc::new(rpc_client),
        })
    }

    pub async fn register(&self) -> Result<(), Error> {
        // Todo: Refactoring `seeder`	https://github.com/radiusxyz/seeder/issues/2
        Ok(())
    }

    pub async fn deregister(&self) -> Result<(), Error> {
        // Todo: Refactoring `seeder`	https://github.com/radiusxyz/seeder/issues/2
        Ok(())
    }

    pub async fn get_sequencer_url_list(
        &self,
        sequencer_address_list: Vec<String>,
    ) -> Result<Vec<(String, String)>, Error> {
        // Todo: Refactoring `seeder`	https://github.com/radiusxyz/seeder/issues/2
        Ok(vec![])
    }
}
