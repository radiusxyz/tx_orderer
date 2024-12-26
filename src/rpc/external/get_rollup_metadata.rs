use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollupMetadata {
    pub rollup_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollupMetadataResponse {
    pub rollup_metadata: RollupMetadata,
}

impl GetRollupMetadata {
    pub const METHOD_NAME: &'static str = "get_rollup_metadata";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetRollupMetadataResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let rollup_metadata = RollupMetadata::get(&parameter.rollup_id)?;

        Ok(GetRollupMetadataResponse { rollup_metadata })
    }
}
