use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithOrderCommitment {
    pub rollup_id: String,
    pub rollup_block_height: u64,
    pub transaction_order: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithOrderCommitmentResponse {
    pub raw_transaction: RawTransaction,
    pub is_direct_sent: bool,
}

impl GetRawTransactionWithOrderCommitment {
    pub const METHOD_NAME: &'static str = "get_raw_transaction_with_order_commitment";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetRawTransactionWithOrderCommitmentResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let (raw_transaction, is_direct_sent) = RawTransactionModel::get(
            &parameter.rollup_id,
            parameter.rollup_block_height,
            parameter.transaction_order,
        )?;

        Ok(GetRawTransactionWithOrderCommitmentResponse {
            raw_transaction,
            is_direct_sent,
        })
    }
}
