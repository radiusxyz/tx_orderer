use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncTransaction {
    pub transaction: UserTransaction,
    pub order_commitment: OrderCommitment,
}

impl SyncTransaction {
    pub const METHOD_NAME: &'static str = stringify!(SyncTransaction);

    pub async fn handler(
        parameter: RpcParameter,
        context: Arc<SsalClient>,
    ) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;
        parameter.transaction.put(
            parameter.order_commitment.rollup_block_number(),
            parameter.order_commitment.transaction_order(),
        )?;

        Ok(())
    }
}
