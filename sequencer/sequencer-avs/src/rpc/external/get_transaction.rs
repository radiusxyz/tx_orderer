use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransaction {
    pub rollup_block_number: RollupBlockNumber,
    pub transaction_order: u64,
}

impl GetTransaction {
    pub const METHOD_NAME: &'static str = stringify!(GetTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<SsalClient>,
    ) -> Result<UserTransaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;
        let transaction =
            UserTransaction::get(parameter.rollup_block_number, parameter.transaction_order)?;

        Ok(transaction)
    }
}
