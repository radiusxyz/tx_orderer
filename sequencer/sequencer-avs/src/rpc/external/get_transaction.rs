use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransaction {
    pub rollup_block_number: u64,
    pub transaction_order: u64,
}

impl GetTransaction {
    pub const METHOD_NAME: &'static str = stringify!(GetTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<AppState>,
    ) -> Result<UserTransaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;
        let database = context.database();

        UserTransaction::get(
            &database,
            parameter.rollup_block_number,
            parameter.transaction_order,
        )
        .map_err(|error| error.into())
    }
}
