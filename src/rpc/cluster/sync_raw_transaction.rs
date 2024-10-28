use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRawTransactionMessage {
    pub rollup_id: String,
    pub rollup_block_height: u64,
    pub transaction_order: u64,
    pub raw_transaction: RawTransaction,
    pub order_commitment: Option<OrderCommitment>,
    pub order_hash: OrderHash,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRawTransaction {
    pub message: SyncRawTransactionMessage,
    pub signature: Signature,
}

impl SyncRawTransaction {
    pub const METHOD_NAME: &'static str = "sync_raw_transaction";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        tracing::info!(
            "Sync raw transaction - rollup id: {:?}, rollup block height: {:?}, transaction order: {:?}, order commitment: {:?}, order hash: {:?}",
            parameter.message.rollup_id,
            parameter.message.rollup_block_height,
            parameter.message.transaction_order,
            parameter.message.order_commitment,
            parameter.message.order_hash,
        );

        let rollup = Rollup::get(&parameter.message.rollup_id)?;
        let mut rollup_metadata = RollupMetadata::get_mut(&parameter.message.rollup_id)?;
        let cluster = Cluster::get(
            rollup.platform(),
            rollup.service_provider(),
            rollup.cluster_id(),
            rollup_metadata.platform_block_height(),
        )?;

        // Verify the leader signature
        let leader_address = cluster.get_leader_address(parameter.message.rollup_block_height)?;
        parameter.signature.verify_message(
            rollup.platform().into(),
            &parameter.message,
            Address::from_str(rollup.platform().into(), &leader_address)?,
        )?;

        // Check the rollup block height
        if parameter.message.rollup_block_height != rollup_metadata.rollup_block_height() {
            return Err(Error::BlockHeightMismatch.into());
        }

        if parameter.message.transaction_order == rollup_metadata.transaction_order() {
            rollup_metadata.increase_transaction_order();
            rollup_metadata
                .update_order_hash(&parameter.message.raw_transaction.raw_transaction_hash());
            rollup_metadata.update()?;
        }

        let transaction_hash = parameter.message.raw_transaction.raw_transaction_hash();

        RawTransactionModel::put_with_transaction_hash(
            &parameter.message.rollup_id,
            &transaction_hash,
            &parameter.message.raw_transaction,
        )?;

        RawTransactionModel::put(
            &parameter.message.rollup_id,
            parameter.message.rollup_block_height,
            parameter.message.transaction_order,
            &parameter.message.raw_transaction,
        )?;

        if let Some(order_commitment) = parameter.message.order_commitment {
            order_commitment.put(
                &parameter.message.rollup_id,
                parameter.message.rollup_block_height,
                parameter.message.transaction_order,
            )?;
        }

        // Temporary block commitment
        BlockCommitment::put(
            &parameter.message.order_hash.into(),
            &parameter.message.rollup_id,
            parameter.message.rollup_block_height,
            parameter.message.transaction_order,
        )?;

        Ok(())
    }
}
