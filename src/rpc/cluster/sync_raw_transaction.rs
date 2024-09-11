use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRawTransaction {
    pub rollup_id: String,
    pub raw_transaction: RawTransaction,
    pub order_commitment: OrderCommitment,
}

impl SyncRawTransaction {
    pub const METHOD_NAME: &'static str = "sync_raw_transaction";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.rollup_id)?;
        let transaction_order = rollup_metadata.issue_transaction_order();
        let rollup_block_height = rollup_metadata.block_height();
        rollup_metadata.update()?;

        EncryptedTransactionModel::unencrypted_transaction();
        let encrypted_transaction_model = EncryptedTransactionModel::unencrypted_transaction();
        encrypted_transaction_model.put(
            &parameter.rollup_id,
            rollup_block_height,
            transaction_order,
        )?;

        let raw_transaction_model = RawTransactionModel::new(parameter.raw_transaction);
        raw_transaction_model.put(&parameter.rollup_id, rollup_block_height, transaction_order)?;

        OrderCommitmentModel::put(
            &parameter.rollup_id,
            rollup_block_height,
            transaction_order,
            &parameter.order_commitment,
        )?;

        Ok(())
    }
}
