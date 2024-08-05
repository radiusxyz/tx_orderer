use crate::{models::SequencingInfoModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddSequencingInfo {
    pub platform: PlatForm,                               // Local / Ethereum
    pub sequencing_function_type: SequencingFunctionType, // Liveness / Validation
    pub service_type: ServiceType,                        // Radius / EigenLayer

    pub provider_rpc_url: IpAddress,
    pub provider_websocket_url: IpAddress,

    pub contract_address: Option<Address>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddSequencingInfoResponse {
    pub success: bool,
}

impl AddSequencingInfo {
    pub const METHOD_NAME: &'static str = "add_sequencing_info";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<AddSequencingInfoResponse, RpcError> {
        let parameter = parameter.parse::<AddSequencingInfo>()?;

        let sequencing_info = SequencingInfo::new(
            parameter.platform.clone(),
            parameter.sequencing_function_type.clone(),
            parameter.service_type.clone(),
            parameter.provider_rpc_url,
            parameter.provider_websocket_url,
            parameter.contract_address,
        );

        // TODO
        if parameter.platform != PlatForm::Local {
            if parameter.sequencing_function_type == SequencingFunctionType::Liveness {
                match parameter.service_type {
                    ServiceType::Radius => {
                        radius_liveness_event_listener::init(sequencing_info.clone());
                    }
                    _ => {}
                }
            }

            if parameter.sequencing_function_type == SequencingFunctionType::Validation {}
        }

        SequencingInfoModel::add(sequencing_info)?;

        Ok(AddSequencingInfoResponse { success: true })
    }
}
