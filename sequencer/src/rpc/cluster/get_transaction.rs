use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransaction {
    pub rollup_id: RollupId,
    pub rollup_block_height: u64,
    pub transaction_order: u64,
}

impl GetTransaction {
    pub const METHOD_NAME: &'static str = stringify!(GetTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<Transaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        Transaction::get(
            parameter.rollup_id,
            parameter.rollup_block_height,
            parameter.transaction_order,
        )
        .map_err(|error| error.into())
    }
}
