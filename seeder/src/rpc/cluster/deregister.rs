use sequencer::{
    models::{LivenessClusterModel, ValidationClusterModel},
    types::{Address, ClusterId, PlatForm, SequencingFunctionType, ServiceType},
};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deregister {
    platform: PlatForm,
    sequencing_function_type: SequencingFunctionType,
    service_type: ServiceType,

    cluster_id: ClusterId,
    address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeregisterResponse {
    pub success: bool,
}

impl Deregister {
    pub const METHOD_NAME: &'static str = "deregister";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<DeregisterResponse, RpcError> {
        let parameter = parameter.parse::<Deregister>()?;

        match parameter.sequencing_function_type {
            SequencingFunctionType::Liveness => {
                let mut liveness_cluster_model = LivenessClusterModel::get_mut(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?;

                liveness_cluster_model
                    .sequencer_addresses
                    .remove(&parameter.address);

                let _ = liveness_cluster_model.update()?;
            }

            SequencingFunctionType::Validation => {
                let mut validation_cluster_model = ValidationClusterModel::get_mut(
                    &parameter.platform,
                    &parameter.service_type,
                    &parameter.cluster_id,
                )?;

                validation_cluster_model
                    .validator_addresses
                    .remove(&parameter.address);

                let _ = validation_cluster_model.update()?;
            }
        }
        Ok(DeregisterResponse { success: true })
    }
}
