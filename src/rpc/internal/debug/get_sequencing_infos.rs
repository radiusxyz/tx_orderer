use serde::{Deserialize, Serialize};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfos;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfosResponse {
    sequencing_infos: Vec<((Platform, ServiceProvider), SequencingInfoPayload)>,
}

impl GetSequencingInfos {
    pub const METHOD_NAME: &'static str = "get_sequencing_infos";

    // pub async fn handler(
    //     _parameter: RpcParameter,
    //     _context: Arc<AppState>,
    // ) -> Result<GetSequencingInfosResponse, RpcError> {
    //     let sequencing_infos = SequencingInfosModel::get_or_default()?
    //         .iter()
    //         .map(|(k, v)| (*k, v.clone()))
    //         .collect();

    //     Ok(GetSequencingInfosResponse { sequencing_infos })
    // }

    pub async fn handler(
        _parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetSequencingInfosResponse, RpcError> {
        let sequencing_info_list = SequencingInfoListModel::get()?;
        let sequencing_infos: Vec<((Platform, ServiceProvider), SequencingInfoPayload)> =
            sequencing_info_list
                .iter()
                .filter_map(|(platform, service_provider)| {
                    if let Some(payload) =
                        SequencingInfoPayloadModel::get(*platform, *service_provider).ok()
                    {
                        Some(((*platform, *service_provider), payload))
                    } else {
                        None
                    }
                })
                .collect();

        Ok(GetSequencingInfosResponse { sequencing_infos })
    }
}
