use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncTransaction {
    pub rollup_id: String,
    pub transaction: Transaction,
    pub order_commitment: OrderCommitment,
}

impl SyncTransaction {
    pub const METHOD_NAME: &'static str = "sync_transaction";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.rollup_id)?;
        rollup_metadata.issue_transaction_order();
        // rollup_metadata.issue_order_hash();
        rollup_metadata.update()?;

        Ok(())
    }
}
