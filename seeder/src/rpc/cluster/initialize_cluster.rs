use sequencer::{
    models::{ClusterIdListModel, LivenessClusterModel, ValidationClusterModel},
    types::{ClusterId, PlatForm, SequencingFunctionType, ServiceType},
};

use crate::rpc::prelude::*;

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
                            parameter.platform.clone(),
                            parameter.service_type.clone(),
                            parameter.cluster_id.clone(),
                        );

                        cluster_model.put()?;
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
                            parameter.platform.clone(),
                            parameter.service_type.clone(),
                            parameter.cluster_id.clone(),
                        );

                        cluster_model.put()?;
                    }
                }
            }
        }

        let mut cluster_id_list_model = ClusterIdListModel::entry(
            &parameter.platform,
            &parameter.sequencing_function_type,
            &parameter.service_type,
        )?;

        cluster_id_list_model.add_cluster_id(parameter.cluster_id.clone());
        cluster_id_list_model.update()?;

        Ok(InitializeClusterResponse { success: true })
    }
}
