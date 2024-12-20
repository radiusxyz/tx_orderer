use serde::{Deserialize, Serialize};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfo {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfoResponse {
    pub sequencing_info_payload: SequencingInfoPayload,
}

impl GetSequencingInfo {
    pub const METHOD_NAME: &'static str = "get_sequencing_info";

    // pub async fn handler(
    //     parameter: RpcParameter,
    //     _context: Arc<AppState>,
    // ) -> Result<GetSequencingInfoResponse, RpcError> {
    //     let parameter = parameter.parse::<GetSequencingInfo>()?;
    //     let sequencing_key = (parameter.platform, parameter.service_provider);

    //     let sequencing_info_payload = SequencingInfosModel::get_or_default()?
    //         .sequencing_infos()
    //         .get(&sequencing_key)
    //         .ok_or(Error::NotFoundSequencingInfo)?
    //         .clone();

    //     Ok(GetSequencingInfoResponse {
    //         sequencing_info_payload,
    //     })
    // }
    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetSequencingInfoResponse, RpcError> {
        let parameter = parameter.parse::<GetSequencingInfo>()?;

        let sequencing_info_payload =
            SequencingInfoPayload::get(parameter.platform, parameter.service_provider)?;

        Ok(GetSequencingInfoResponse {
            sequencing_info_payload,
        })
    }
}
