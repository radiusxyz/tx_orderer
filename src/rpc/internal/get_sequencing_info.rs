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

impl RpcParameter<AppState> for GetSequencingInfo {
    type Response = GetSequencingInfoResponse;

    fn method() -> &'static str {
        "get_sequencing_info"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let sequencing_info_payload =
            SequencingInfoPayload::get(self.platform, self.service_provider)?;

        Ok(GetSequencingInfoResponse {
            sequencing_info_payload,
        })
    }
}
