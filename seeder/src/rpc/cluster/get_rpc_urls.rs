use std::collections::HashMap;

use sequencer::types::{
    Address, AddressList, ClusterId, IpAddress, PlatForm, SequencingFunctionType, ServiceType,
};
use tracing::info;

use crate::{
    models::{LivenessClusterModel, SequencerModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRpcUrls {
    pub platform: PlatForm,
    pub sequencing_function_type: SequencingFunctionType,
    pub service_type: ServiceType,

    pub cluster_id: ClusterId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRpcUrlsResponse {
    pub rpc_urls: HashMap<Address, IpAddress>,
}

impl GetRpcUrls {
    pub const METHOD_NAME: &'static str = "get_rpc_urls";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<GetRpcUrlsResponse, RpcError> {
        let parameter = parameter.parse::<GetRpcUrls>()?;

        info!("get_rpc_urls: {:?}", parameter.cluster_id);

        // TODO:
        let address_list = match parameter.sequencing_function_type {
            SequencingFunctionType::Liveness => {
                let cluster_model = LivenessClusterModel::get(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?;
                cluster_model
                    .sequencer_addresses
                    .keys()
                    .cloned()
                    .collect::<AddressList>()
            }
            SequencingFunctionType::Validation => {
                let cluster_model = LivenessClusterModel::get(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?;
                cluster_model
                    .sequencer_addresses
                    .keys()
                    .cloned()
                    .collect::<AddressList>()
            }
        };

        println!("address_list: {:?}", address_list);

        let rpc_urls = address_list
            .iter()
            .filter_map(
                |sequencer_address| match SequencerModel::get(sequencer_address.clone()) {
                    Ok(sequencer_model) => {
                        if let Some(rpc_url) = sequencer_model.rpc_url {
                            Some((sequencer_model.address, rpc_url))
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                },
            )
            .collect::<HashMap<Address, IpAddress>>();

        Ok(GetRpcUrlsResponse { rpc_urls })
    }
}
