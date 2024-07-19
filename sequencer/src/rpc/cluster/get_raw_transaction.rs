use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransaction {
    pub rollup_block_number: u64,
    pub transaction_order: u64,
}

impl GetRawTransaction {
    pub const METHOD_NAME: &'static str = stringify!(GetRawTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<UserRawTransaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        UserRawTransaction::get(parameter.rollup_block_number, parameter.transaction_order)
            .map_err(|error| error.into())
    }
}
