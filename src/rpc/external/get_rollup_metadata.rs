use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollupMetadata {
    pub rollup_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollupMetadataResponse {
    pub rollup_metadata: RollupMetadata,
}

impl RpcParameter<AppState> for GetRollupMetadata {
    type Response = GetRollupMetadataResponse;

    fn method() -> &'static str {
        "get_rollup_metadata"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let rollup_metadata = RollupMetadata::get(&self.rollup_id)?;

        tracing::info!(
            "Get rollup metadata: rollup_id: {:?}, rollup_metadata: {:?}",
            self.rollup_id,
            rollup_metadata.clone()
        );

        Ok(GetRollupMetadataResponse { rollup_metadata })
    }
}
