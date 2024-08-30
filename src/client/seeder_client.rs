use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::json_rpc::{Error as JsonRpcError, ErrorKind, RpcClient};
use serde::{Deserialize, Serialize};
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
struct GetRpcUrlListResponse {
    pub rpc_url_list: Vec<(Address, IpAddress)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GetRpcUrlResponse {
    pub rpc_url: IpAddress,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GetSequencingInfosResponse {
    // todo: change string to type
    sequencing_infos: HashMap<String, SequencingInfo>,
}

impl SeederClient {
    pub fn new(seeder_rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let client = RpcClient::new(seeder_rpc_url).map_err(|error| {
            Error::RpcError(JsonRpcError::custom(ErrorKind::BuildClient, error))
        })?;

        Ok(Self(Arc::new(client)))
    }

    // pub async fn register_rpc_url(
    //     &self,
    //     address: Address,
    //     rpc_url: IpAddress,
    // ) -> Result<(), Error> {
    //     let rpc_method = json!({
    //         "address": address,
    //         "rpc_url": rpc_url,
    //     });

    //     info!("Get register_rpc_url - rpc_method: {:?}", rpc_method);

    //     let register_rpc_url_response: RegisterRpcUrlResponse =
    //         self.0.request("register_rpc_url", rpc_method).await?;

    //     if !register_rpc_url_response.success {
    //         return Err(Error::RegisterRpcUrl);
    //     }

    //     Ok(())
    // }

    // pub async fn get_rpc_url_list(
    //     &self,
    //     platform: &Platform,
    //     sequencing_function_type: &SequencingFunctionType,
    //     service_type: &ServiceType,
    //     cluster_id: &ClusterId,
    // ) -> Result<Vec<(Address, IpAddress)>, Error> {
    //     let rpc_method = json!({
    //       "platform": platform,
    //       "sequencing_function_type": sequencing_function_type,
    //       "service_type": service_type,
    //       "cluster_id": cluster_id
    //     });

    //     info!("Get rpc urls - rpc_method: {:?}", rpc_method);

    //     let get_rpc_url_list_response: GetRpcUrlListResponse =
    //         self.0.request("get_rpc_url_list", rpc_method).await?;

    //     Ok(get_rpc_url_list_response.rpc_url_list)
    // }

    // pub async fn get_rpc_url(&self, address: &Address) -> Result<IpAddress, Error> {
    //     let rpc_method = json!({
    //       "address": address,
    //     });

    //     info!("Get rpc urls - rpc_method: {:?}", rpc_method);

    //     let get_rpc_url_response: GetRpcUrlResponse =
    //         self.0.request("get_rpc_url", rpc_method).await?;

    //     Ok(get_rpc_url_response.rpc_url)
    // }

    // // todo(jaemin): remove get_sequencing_infos
    // pub async fn get_sequencing_infos(&self) -> Result<HashMap<String, SequencingInfo>, Error> {
    //     let rpc_method = json!({});

    //     info!("Get sequencing infos - rpc_method: {:?}", rpc_method);

    //     let get_sequencing_infos_response: GetSequencingInfosResponse =
    //         self.0.request("get_sequencing_infos", rpc_method).await?;

    //     Ok(get_sequencing_infos_response.sequencing_infos)
    // }
}
