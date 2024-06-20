use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncUserTransaction {
    pub transaction: UserTransaction,
    pub order_commitment: OrderCommitment,
}

impl SyncUserTransaction {
    pub const METHOD_NAME: &'static str = stringify!(SyncUserTransaction);

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;
        let database = context.database();

        parameter.transaction.put(
            &database,
            parameter.order_commitment.rollup_block_number(),
            parameter.order_commitment.transaction_order(),
        )?;

        Ok(())
    }
}
