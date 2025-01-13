use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollup {
    pub rollup_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRollupResponse {
    pub rollup: Rollup,
}

impl RpcParameter<AppState> for GetRollup {
    type Response = GetRollupResponse;

    fn method() -> &'static str {
        "get_rollup"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let rollup = Rollup::get(&self.rollup_id)?;

        Ok(GetRollupResponse { rollup })
    }
}
