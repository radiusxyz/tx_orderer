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

impl GetOrderCommitment {
    pub const METHOD_NAME: &'static str = "get_order_commitment";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetOrderCommitmentResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let order_commitment = OrderCommitment::get(
            &parameter.rollup_id,
            parameter.rollup_block_height,
            parameter.transaction_order,
        )?;

        Ok(GetOrderCommitmentResponse { order_commitment })
    }
}
