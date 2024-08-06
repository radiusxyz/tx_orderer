use crate::{
    models::{RollupMetadataModel, TransactionModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncTransaction {
    pub rollup_id: ClusterId,
    pub transaction: TransactionModel,
    pub order_commitment: OrderCommitment,
}

impl SyncTransaction {
    pub const METHOD_NAME: &'static str = stringify!(SyncRequest);

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.rollup_id)?;

        // TODO: compare block height and transaction order with order commitment
        rollup_metadata.increment_transaction_order();
        rollup_metadata.update()?;

        parameter.transaction.put(
            &parameter.rollup_id,
            &parameter.order_commitment.data.block_height,
            &parameter.order_commitment.data.transaction_order,
        )?;

        Ok(())
    }
}
