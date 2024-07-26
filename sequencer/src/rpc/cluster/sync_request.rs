use crate::{
    models::{ClusterMetadataModel, TransactionModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRequest {
    pub rollup_id: RollupId,
    pub transaction: TransactionModel,
    pub order_commitment: OrderCommitment,
}

impl SyncRequest {
    pub const METHOD_NAME: &'static str = stringify!(SyncRequest);

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut cluster_metadata = ClusterMetadataModel::get_mut()?;

        // TODO: compare block height and transaction order with order commitment
        cluster_metadata.transaction_order.increment();
        cluster_metadata.commit()?;

        parameter.transaction.put(
            &parameter.rollup_id,
            &parameter.order_commitment.data.block_height,
            &parameter.order_commitment.data.transaction_order,
        )?;

        Ok(())
    }
}
