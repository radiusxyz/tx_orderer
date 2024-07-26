use crate::{
    models::{ClusterMetadataModel, TransactionModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRequest {
    pub rollup_id: RollupId,
    pub block_height: BlockHeight,
    pub transaction_order: TransactionOrder,
    pub transaction_model: TransactionModel,
}

impl SyncRequest {
    pub const METHOD_NAME: &'static str = stringify!(SyncRequest);

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut cluster_metadata = ClusterMetadataModel::get_mut()?;

        cluster_metadata.transaction_order.increment();
        cluster_metadata.commit()?;

        parameter.transaction_model.put(
            &parameter.rollup_id,
            &parameter.block_height,
            &parameter.transaction_order,
        )?;

        Ok(())
    }
}
