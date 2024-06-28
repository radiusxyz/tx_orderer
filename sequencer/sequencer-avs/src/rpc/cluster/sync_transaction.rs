use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncTransaction {
    pub transaction: UserTransaction,
    pub order_commitment: OrderCommitment,
}

impl SyncTransaction {
    pub const METHOD_NAME: &'static str = stringify!(SyncTransaction);

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut cluster_metadata = ClusterMetadata::get_mut()?;
        cluster_metadata.transaction_order += 1;
        cluster_metadata.commit()?;

        parameter.transaction.put(
            parameter.order_commitment.rollup_block_number,
            parameter.order_commitment.transaction_order,
        )?;

        Ok(())
    }
}
