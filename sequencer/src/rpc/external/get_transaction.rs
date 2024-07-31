use crate::{models::TransactionModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransaction {
    pub rollup_id: RollupId,
    pub rollup_block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
}

impl GetTransaction {
    pub const METHOD_NAME: &'static str = stringify!(GetTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<TransactionModel, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        TransactionModel::get(
            &parameter.rollup_id,
            &parameter.rollup_block_height,
            &parameter.transaction_order,
        )
        .map_err(|error| error.into())
    }
}
