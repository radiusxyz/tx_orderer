use sequencer::{
    models::{LivenessClusterModel, ValidationClusterModel},
    types::{Address, ClusterId, IpAddress, PlatForm, SequencingFunctionType, ServiceType},
};
use tracing::info;

use crate::{models::OperatorModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRpcUrlList {
    pub platform: PlatForm,
    pub sequencing_function_type: SequencingFunctionType,
    pub service_type: ServiceType,

    pub cluster_id: ClusterId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRpcUrlListResponse {
    pub rpc_url_list: Vec<(Address, IpAddress)>,
}

impl GetRpcUrlList {
    pub const METHOD_NAME: &'static str = "get_rpc_url_list";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<GetRpcUrlListResponse, RpcError> {
        let parameter = parameter.parse::<GetRpcUrlList>()?;

        info!("get_rpc_url_list: {:?}", parameter.cluster_id);

        let address_list = match parameter.sequencing_function_type {
            SequencingFunctionType::Liveness => {
                LivenessClusterModel::get(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?
                .sequencer_address_list
            }
            SequencingFunctionType::Validation => {
                ValidationClusterModel::get(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?
                .validator_address_list
            }
        };

        println!("address_list: {:?}", address_list);

        let rpc_url_list: Vec<(Address, IpAddress)> = address_list
            .iter()
            .filter_map(|operator_address| {
                OperatorModel::get(operator_address.clone())
                    .ok()
                    .and_then(|operator_model| {
                        operator_model
                            .rpc_url
                            .map(|rpc_url| (operator_model.address, rpc_url))
                    })
            })
            .collect();

        Ok(GetRpcUrlListResponse { rpc_url_list })
    }
}
