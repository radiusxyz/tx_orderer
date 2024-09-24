use crate::{rpc::prelude::*, types::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithOrderCommitment {
    rollup_id: String,
    rollup_block_height: u64,
    transaction_order: u64,
}

impl GetRawTransactionWithOrderCommitment {
    pub const METHOD_NAME: &'static str = "get_raw_transaction_with_order_commitment";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<RawTransaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let encrypted_transaction = RawTransactionModel::get(
            &parameter.rollup_id,
            parameter.rollup_block_height,
            parameter.transaction_order,
        )?;

        Ok(encrypted_transaction)
    }
}
