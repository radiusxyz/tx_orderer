use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::json_rpc::{Error as JsonRpcError, ErrorKind, RpcClient};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::{error::Error, types::*};

pub struct SeederClient(Arc<RpcClient>);

unsafe impl Send for SeederClient {}

unsafe impl Sync for SeederClient {}

impl Clone for SeederClient {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RegisterRpcUrlResponse {
    pub success: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GetRpcUrlsResponse {
    pub rpc_urls: HashMap<Address, IpAddress>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GetRpcUrlResponse {
    pub rpc_url: IpAddress,
}

impl SeederClient {
    pub fn new(seeder_rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let client = RpcClient::new(seeder_rpc_url).map_err(|error| {
            Error::RpcError(JsonRpcError::custom(ErrorKind::BuildClient, error))
        })?;

        Ok(Self(Arc::new(client)))
    }

    pub async fn register_rpc_url(
        &self,
        address: Address,
        rpc_url: IpAddress,
    ) -> Result<(), Error> {
        let rpc_method = json!({
            "address": address,
            "rpc_url": rpc_url,
        });

        info!("Get register_rpc_url - rpc_method: {:?}", rpc_method);

        let register_rpc_url_response: RegisterRpcUrlResponse =
            self.0.request("register_rpc_url", rpc_method).await?;

        if !register_rpc_url_response.success {
            return Err(Error::RegisterRpcUrl);
        }

        Ok(())
    }

    pub async fn get_rpc_urls(
        &self,
        platform: &PlatForm,
        sequencing_function_type: &SequencingFunctionType,
        service_type: &ServiceType,
        cluster_id: &ClusterId,
    ) -> Result<HashMap<Address, IpAddress>, Error> {
        let rpc_method = json!({
          "platform": platform,
          "sequencing_function_type": sequencing_function_type,
          "service_type": service_type,
          "cluster_id": cluster_id
        });

        info!("Get rpc urls - rpc_method: {:?}", rpc_method);

        let get_rpc_urls_response: GetRpcUrlsResponse =
            self.0.request("get_rpc_urls", rpc_method).await?;

        Ok(get_rpc_urls_response.rpc_urls)
    }

    pub async fn get_rpc_url(&self, address: &Address) -> Result<IpAddress, Error> {
        let rpc_method = json!({
          "address": address,
        });

        info!("Get rpc urls - rpc_method: {:?}", rpc_method);

        let get_rpc_url_response: GetRpcUrlResponse =
            self.0.request("get_rpc_url", rpc_method).await?;

        Ok(get_rpc_url_response.rpc_url)
    }
}
