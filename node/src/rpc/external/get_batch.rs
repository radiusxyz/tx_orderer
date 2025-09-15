use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBatch {
    pub rollup_id: RollupId,
    pub batch_number: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBatchResponse {
    pub batch: Batch,
}

impl RpcParameter<AppState> for GetBatch {
    type Response = GetBatchResponse;

    fn method() -> &'static str {
        "get_batch"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let batch = Batch::get(&self.rollup_id, self.batch_number)?;

        Ok(GetBatchResponse { batch })
    }
}
