use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTransaction {
    rollup_block_number: RollupBlockNumber,
    transaction_order: u64,
}

impl GetTransaction {
    const METHOD_NAME: &'static str = stringify!(GetTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<()>,
    ) -> Result<Transaction, RpcError> {
        let parameter = parameter.parse::<Self>()?;
        let transaction =
            Transaction::get(parameter.rollup_block_number, parameter.transaction_order)?;

        Ok(transaction)
    }
}
