use sequencer::types::{Address, ClusterType, IpAddress};

use super::prelude::*;
use crate::{
    models::{LivenessInfo, LivenessModel},
    task::event_listener,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddSupportLiveness {
    pub provider_rpc_url: IpAddress,
    pub provider_websocket_url: IpAddress,
    pub liveness_contract_address: Option<Address>,
    pub cluster_type: ClusterType,
}

impl AddSupportLiveness {
    pub const METHOD_NAME: &'static str = "add_support_liveness";
}

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<AddSupportLiveness>()?;

    let liveness_info = LivenessInfo {
        provider_rpc_url: parameter.provider_rpc_url,
        provider_websocket_url: parameter.provider_websocket_url,
        liveness_contract_address: parameter.liveness_contract_address,
        cluster_type: parameter.cluster_type,
    };

    match liveness_info.cluster_type {
        ClusterType::Local => {}
        ClusterType::EigenLayer => {
            let liveness_contract_address =
                liveness_info.liveness_contract_address.clone().unwrap();

            event_listener::init(
                liveness_info.provider_websocket_url.to_string(),
                liveness_contract_address.to_string(),
            );
        }
    }

    LivenessModel::add(liveness_info)?;

    Ok(())
}
