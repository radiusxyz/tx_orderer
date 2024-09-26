use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncEncryptedTransaction {
    pub message: SyncEncryptedTransactionMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncEncryptedTransactionMessage {
    pub rollup_id: String,
    pub rollup_block_height: u64,
    pub transaction_order: u64,
    pub encrypted_transaction: EncryptedTransaction,
    pub order_commitment: OrderCommitment,
}

impl SyncEncryptedTransaction {
    pub const METHOD_NAME: &'static str = "sync_encrypted_transaction";

    pub async fn handler(parameter: RpcParameter, _context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        let rollup = RollupModel::get(&parameter.message.rollup_id)?;
        let mut rollup_metadata = RollupMetadataModel::get_mut(&parameter.message.rollup_id)?;
        let cluster = ClusterModel::get(
            rollup.platform(),
            rollup.service_provider(),
            rollup.cluster_id(),
            rollup_metadata.platform_block_height(),
        )?;

        // Verify the leader signature
        parameter.signature.verify_message(
            rollup.platform().into(),
            &parameter.message,
            cluster.get_leader_address(parameter.message.rollup_block_height)?,
        )?;

        rollup_metadata.increase_transaction_order();
        rollup_metadata.update()?;

        let transaction_hash = parameter
            .message
            .encrypted_transaction
            .raw_transaction_hash();

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

        OrderCommitmentModel::put(
            &parameter.message.rollup_id,
            parameter.message.rollup_block_height,
            parameter.message.transaction_order,
            &parameter.message.order_commitment,
        )?;

        Ok(())
    }
}
