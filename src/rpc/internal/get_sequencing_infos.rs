use serde::{Deserialize, Serialize};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfos;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfosResponse {
    pub sequencing_infos: Vec<((Platform, LivenessServiceProvider), SequencingInfoPayload)>,
}

impl RpcParameter<AppState> for GetSequencingInfos {
    type Response = GetSequencingInfosResponse;

    fn method() -> &'static str {
        "get_sequencing_infos"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let sequencing_info_list = SequencingInfoList::get()?;

        let sequencing_infos: Vec<((Platform, LivenessServiceProvider), SequencingInfoPayload)> =
            sequencing_info_list
                .iter()
                .filter_map(|(platform, liveness_service_provider)| {
                    SequencingInfoPayload::get(*platform, *liveness_service_provider)
                        .ok()
                        .map(|payload| ((*platform, *liveness_service_provider), payload))
                })
                .collect();

        Ok(GetSequencingInfosResponse { sequencing_infos })
    }
}
