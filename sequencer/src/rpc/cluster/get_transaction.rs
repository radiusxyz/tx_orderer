use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransaction {
    pub full_node_id: u32,
    pub rollup_block_number: u64,
    pub transaction_order: u64,
}

impl GetTransaction {
    pub const METHOD_NAME: &'static str = stringify!(GetTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<UserTransaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        UserTransaction::get(
            parameter.full_node_id,
            parameter.rollup_block_number,
            parameter.transaction_order,
        )
        .map_err(|error| error.into())
    }
}
