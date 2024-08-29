use crate::{models::RawTransactionModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransaction {
    pub rollup_id: RollupId,
    pub block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionResponse {
    raw_transaction: RawTransaction,
}

impl GetRawTransaction {
    pub const METHOD_NAME: &'static str = "get_raw_transaction";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<GetRawTransactionResponse, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let raw_transaction_model = RawTransactionModel::get(
            &parameter.rollup_id,
            &parameter.block_height,
            &parameter.transaction_order,
        )?;

        Ok(GetRawTransactionResponse {
            raw_transaction: raw_transaction_model.raw_transaction().clone(),
        })
    }
}
