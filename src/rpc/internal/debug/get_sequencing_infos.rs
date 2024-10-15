use serde::{Deserialize, Serialize};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfos;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfosResponse {
    pub sequencing_infos: Vec<((Platform, ServiceProvider), SequencingInfoPayload)>,
}

impl GetSequencingInfos {
    pub const METHOD_NAME: &'static str = "get_sequencing_infos";

    pub async fn handler(
        _parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetSequencingInfosResponse, RpcError> {
        let sequencing_info_list = SequencingInfoList::get()?;

        let sequencing_infos: Vec<((Platform, ServiceProvider), SequencingInfoPayload)> =
            sequencing_info_list
                .iter()
                .filter_map(|(platform, service_provider)| {
                    SequencingInfoPayload::get(*platform, *service_provider)
                        .ok()
                        .map(|payload| ((*platform, *service_provider), payload))
                })
                .collect();

        Ok(GetSequencingInfosResponse { sequencing_infos })
    }
}
