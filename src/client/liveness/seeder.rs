use std::sync::Arc;

use radius_sequencer_sdk::json_rpc::{Error, RpcClient};

use crate::types::*;

/// 09/05
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

    pub async fn get_cluster_info(
        &self,
        platform: Platform,
        service_provider: ServiceProvider,
        cluster_id: String,
        sequencer_address_list: Vec<String>,
    ) -> Result<ClusterInfo, Error> {
        // Todo: Refactoring `seeder`	https://github.com/radiusxyz/seeder/issues/2
        todo!("ClusterInfo");
    }
}
