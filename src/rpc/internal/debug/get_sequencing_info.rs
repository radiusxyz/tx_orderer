use serde::{Deserialize, Serialize};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfo {
    platform: Platform,
    service_provider: ServiceProvider,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfoResponse {
    sequencing_info_payload: SequencingInfoPayload,
}

impl GetSequencingInfo {
    pub const METHOD_NAME: &'static str = "get_sequencing_info";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetSequencingInfoResponse, RpcError> {
        let parameter = parameter.parse::<GetSequencingInfo>()?;
        let sequencing_key = (parameter.platform, parameter.service_provider);

        let sequencing_info_payload = SequencingInfosModel::get()?
            .sequencing_infos()
            .get(&sequencing_key)
            .ok_or(Error::NotFoundSequencingInfo)?
            .clone();

        Ok(GetSequencingInfoResponse {
            sequencing_info_payload,
        })
    }
}
