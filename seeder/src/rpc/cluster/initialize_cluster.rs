use sequencer::types::{ClusterId, PlatForm, SequencingFunctionType, ServiceType};

use crate::{
    models::{LivenessClusterModel, ValidationClusterModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitializeCluster {
    platform: PlatForm,
    sequencing_function_type: SequencingFunctionType,
    service_type: ServiceType,

    cluster_id: ClusterId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitializeClusterResponse {
    pub success: bool,
}

impl InitializeCluster {
    pub const METHOD_NAME: &'static str = "initialize_cluster";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<InitializeClusterResponse, RpcError> {
        let parameter = parameter.parse::<InitializeCluster>()?;

        match parameter.sequencing_function_type {
            SequencingFunctionType::Liveness => {
                match LivenessClusterModel::get(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                ) {
                    Ok(_) => {}
                    Err(_) => {
                        let cluster_model = LivenessClusterModel::new(
                            parameter.platform,
                            parameter.service_type,
                            parameter.cluster_id,
                        );

                        let _ = cluster_model.put()?;
                    }
                }
            }
            SequencingFunctionType::Validation => {
                match ValidationClusterModel::get(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                ) {
                    Ok(_) => {}
                    Err(_) => {
                        let cluster_model = ValidationClusterModel::new(
                            parameter.platform,
                            parameter.service_type,
                            parameter.cluster_id,
                        );

                        let _ = cluster_model.put()?;
                    }
                }
            }
        }

        Ok(InitializeClusterResponse { success: true })
    }
}
