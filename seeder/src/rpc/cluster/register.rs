use sequencer::{
    models::{LivenessClusterModel, ValidationClusterModel},
    types::{Address, ClusterId, PlatForm, SequencingFunctionType, ServiceType},
};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Register {
    platform: PlatForm,
    sequencing_function_type: SequencingFunctionType,
    service_type: ServiceType,

    cluster_id: ClusterId,
    address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
}

impl Register {
    pub const METHOD_NAME: &'static str = "register";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<RegisterResponse, RpcError> {
        let parameter = parameter.parse::<Register>()?;

        match parameter.sequencing_function_type {
            SequencingFunctionType::Liveness => {
                let mut liveness_cluster_model = LivenessClusterModel::get_mut(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?;

                liveness_cluster_model.add_seqeuncer(parameter.address);
                liveness_cluster_model.update()?;
            }

            SequencingFunctionType::Validation => {
                let mut validation_cluster_model = ValidationClusterModel::get_mut(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?;

                validation_cluster_model.add_seqeuncer(parameter.address);
                validation_cluster_model.update()?;
            }
        }
        Ok(RegisterResponse { success: true })
    }
}
