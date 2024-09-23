use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncEncryptedTransaction {
    pub rollup_id: String,
    pub encrypted_transaction: EncryptedTransaction,

    pub rollup_block_height: u64,
    pub transaction_order: u64,

    pub order_commitment: OrderCommitment,
}

impl SyncEncryptedTransaction {
    pub const METHOD_NAME: &'static str = "sync_encrypted_transaction";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.rollup_id)?;

        // Check block height
        if parameter.rollup_block_height != rollup_metadata.rollup_block_height() {
            return Err(Error::BlockHeightMismatch.into());
        }

        // TODO: sync??
        if parameter.transaction_order == rollup_metadata.transaction_order() {
            rollup_metadata.increase_transaction_order();
            rollup_metadata
                .update_order_hash(&parameter.encrypted_transaction.raw_transaction_hash());
            rollup_metadata.update()?;
        }

        let transaction_hash = parameter.encrypted_transaction.raw_transaction_hash();
        EncryptedTransactionModel::put_with_transaction_hash(
            &parameter.rollup_id,
            &transaction_hash.inner().to_string(),
            &parameter.encrypted_transaction,
        )?;

        EncryptedTransactionModel::put(
            &parameter.rollup_id,
            parameter.rollup_block_height,
            parameter.transaction_order,
            &parameter.encrypted_transaction,
        )?;

        OrderCommitmentModel::put(
            &parameter.rollup_id,
            parameter.rollup_block_height,
            parameter.transaction_order,
            &parameter.order_commitment,
        )?;

        Ok(())
    }
}
