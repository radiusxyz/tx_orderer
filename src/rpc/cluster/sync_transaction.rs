use crate::{
    models::{RollupMetadataModel, TransactionModel},
    rpc::prelude::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncTransaction {
    pub rollup_id: RollupId,
    pub transaction: TransactionModel,
    pub order_commitment: OrderCommitment,
}

impl SyncTransaction {
    pub const METHOD_NAME: &'static str = "sync_transaction";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut rollup_metadata_model = RollupMetadataModel::get_mut(&parameter.rollup_id)?;

        let rollup_metadata = rollup_metadata_model.rollup_metadata();
        let block_height = rollup_metadata.block_height();
        let transaction_order = parameter.order_commitment.data.transaction_order;

        let new_order_hash = parameter.order_commitment.data.previous_order_hash;

        // TODO check
        if rollup_metadata.order_hash() != &new_order_hash
            && rollup_metadata.transaction_order() == transaction_order
        {
            let mut new_rollup_metadata_model = rollup_metadata_model.rollup_metadata().clone();
            new_rollup_metadata_model.update_order_hash(new_order_hash);
            new_rollup_metadata_model.increase_transaction_order();

            rollup_metadata_model.update_rollup_metadata(new_rollup_metadata_model.clone());
            rollup_metadata_model.update()?;
        }

        match parameter.transaction {
            TransactionModel::Raw(raw_transaction_model) => {
                raw_transaction_model.put(
                    &parameter.rollup_id,
                    &block_height,
                    &transaction_order,
                )?;
            }
            TransactionModel::Encrypted(encrypted_transaction) => {
                encrypted_transaction.put(
                    &parameter.rollup_id,
                    &block_height,
                    &transaction_order,
                )?;
            }
        }

        Ok(())
    }
}
