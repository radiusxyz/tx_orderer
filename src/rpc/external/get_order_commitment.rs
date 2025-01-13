use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetOrderCommitment {
    pub rollup_id: String,
    pub rollup_block_height: u64,
    pub transaction_order: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetOrderCommitmentResponse {
    pub order_commitment: OrderCommitment,
}

impl RpcParameter<AppState> for GetOrderCommitment {
    type Response = GetOrderCommitmentResponse;

    fn method() -> &'static str {
        "get_order_commitment"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let order_commitment = OrderCommitment::get(
            &self.rollup_id,
            self.rollup_block_height,
            self.transaction_order,
        )?;

        Ok(GetOrderCommitmentResponse { order_commitment })
    }
}
