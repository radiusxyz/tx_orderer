use sequencer::types::{ClusterId, PlatForm, SequencingFunctionType, ServiceType};

use crate::{
    models::{ClusterModel, LivenessClusterModel, ValidationClusterModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCluster {
    platform: PlatForm,
    sequencing_function_type: SequencingFunctionType,
    service_type: ServiceType,

    cluster_id: ClusterId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetClusterResponse {
    cluster: ClusterModel,
}

impl GetCluster {
    pub const METHOD_NAME: &'static str = "get_cluster";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<GetClusterResponse, RpcError> {
        let parameter = parameter.parse::<GetCluster>()?;

        let cluster_model = match parameter.sequencing_function_type {
            SequencingFunctionType::Liveness => ClusterModel::Liveness(LivenessClusterModel::get(
                &parameter.platform,
                &parameter.service_type,
                &parameter.cluster_id,
            )?),
            SequencingFunctionType::Validation => {
                ClusterModel::Validation(ValidationClusterModel::get(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?)
            }
        };

        Ok(GetClusterResponse {
            cluster: cluster_model,
        })
    }
}
