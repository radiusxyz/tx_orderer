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
                let _ = liveness_cluster_model.update()?;
            }

            SequencingFunctionType::Validation => {
                let mut validation_cluster_model = ValidationClusterModel::get_mut(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?;

                let is_exist_validator_address = validation_cluster_model
                    .validator_address_list
                    .contains(&parameter.address);

                if !is_exist_validator_address {
                    validation_cluster_model
                        .validator_address_list
                        .push(parameter.address);
                    let _ = validation_cluster_model.update()?;
                }
            }
        }
        Ok(RegisterResponse { success: true })
    }
}
