use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRawTransaction {
    pub rollup_id: String,
    pub raw_transaction: RawTransaction,

    pub rollup_block_height: u64,
    pub transaction_order: u64,

    pub order_commitment: Option<OrderCommitment>,
}

impl SyncRawTransaction {
    pub const METHOD_NAME: &'static str = "sync_raw_transaction";

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
            rollup_metadata.update_order_hash(&parameter.raw_transaction.raw_transaction_hash());
            rollup_metadata.update()?;
        }

        let transaction_hash = parameter.raw_transaction.raw_transaction_hash();
        RawTransactionModel::put_with_transaction_hash(
            &parameter.rollup_id,
            &transaction_hash.inner().to_string(),
            &parameter.raw_transaction,
        )?;

        RawTransactionModel::put(
            &parameter.rollup_id,
            parameter.rollup_block_height,
            parameter.transaction_order,
            &parameter.raw_transaction,
        )?;

        if parameter.order_commitment.is_some() {
            OrderCommitmentModel::put(
                &parameter.rollup_id,
                parameter.rollup_block_height,
                parameter.transaction_order,
                &parameter.order_commitment.unwrap(),
            )?;
        }

        Ok(())
    }
}
