use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRequest {
    pub full_node_id: u32,
    pub transaction: UserTransaction,
    pub order_commitment: OrderCommitment,
}

impl SyncRequest {
    pub const METHOD_NAME: &'static str = stringify!(SyncRequest);

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut cluster_metadata = ClusterMetadata::get_mut()?;
        cluster_metadata.transaction_order += 1;
        cluster_metadata.commit()?;

        parameter.transaction.put(
            parameter.full_node_id,
            parameter.order_commitment.rollup_block_number,
            parameter.order_commitment.transaction_order,
        )?;

        Ok(())
    }
}
