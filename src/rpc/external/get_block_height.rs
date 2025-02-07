use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlockHeight {
    pub rollup_id: String,
}

impl RpcParameter<AppState> for GetBlockHeight {
    type Response = u64;

    fn method() -> &'static str {
        "get_block_height"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let rollup_metadata = RollupMetadata::get(&self.rollup_id)?;

        Ok(rollup_metadata.rollup_block_height - 1)
    }
}
