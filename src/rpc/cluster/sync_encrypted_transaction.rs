use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncEncryptedTransactionMessage {
    pub rollup_id: String,
    pub rollup_block_height: u64,
    pub transaction_order: u64,
    pub encrypted_transaction: EncryptedTransaction,
    pub order_commitment: OrderCommitment,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncEncryptedTransaction {
    pub message: SyncEncryptedTransactionMessage,
    pub signature: Signature,
}

impl SyncEncryptedTransaction {
    pub const METHOD_NAME: &'static str = "sync_encrypted_transaction";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        tracing::info!("sync encrypted transaction - {:?}", parameter);
        tracing::info!(
            "Sync encrypted transaction - rollup id: {:?}, rollup block height: {:?}, transaction order: {:?}, order commitment: {:?}",
            parameter.message.rollup_id,
            parameter.message.rollup_block_height,
            parameter.message.transaction_order,
            parameter.message.order_commitment,
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

        let transaction_hash = parameter
            .message
            .encrypted_transaction
            .raw_transaction_hash();

        rollup_metadata.add_transaction_hash(transaction_hash.as_ref());
        rollup_metadata.update()?;

        EncryptedTransactionModel::put_with_transaction_hash(
            &parameter.message.rollup_id,
            &transaction_hash,
            &parameter.message.encrypted_transaction,
        )?;

        EncryptedTransactionModel::put(
            &parameter.message.rollup_id,
            parameter.message.rollup_block_height,
            parameter.message.transaction_order,
            &parameter.message.encrypted_transaction,
        )?;

        parameter.message.order_commitment.put(
            &parameter.message.rollup_id,
            parameter.message.rollup_block_height,
            parameter.message.transaction_order,
        )?;

        Ok(())
    }
}
