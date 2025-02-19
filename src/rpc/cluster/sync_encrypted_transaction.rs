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

impl RpcParameter<AppState> for SyncEncryptedTransaction {
    type Response = ();

    fn method() -> &'static str {
        "sync_encrypted_transaction"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        tracing::info!(
            "Sync encrypted transaction - rollup id: {:?}, rollup block height: {:?}, transaction order: {:?}, order commitment: {:?}",
            self.message.rollup_id,
            self.message.rollup_block_height,
            self.message.transaction_order,
            self.message.order_commitment,
        );

        let transaction_gas_limit = self
            .message
            .encrypted_transaction
            .get_transaction_gas_limit()?;

        let rollup = Rollup::get(&self.message.rollup_id)?;
        let mut rollup_metadata = RollupMetadata::get_mut(&self.message.rollup_id)?;

        // Verify the leader signature
        let leader_address = &rollup_metadata.leader_tx_orderer_rpc_info.address;
        self.signature
            .verify_message(rollup.platform.into(), &self.message, leader_address)?;

        // Check the rollup block height
        if self.message.rollup_block_height != rollup_metadata.rollup_block_height {
            return Err(Error::BlockHeightMismatch.into());
        }

        let transaction_hash = self.message.encrypted_transaction.raw_transaction_hash();

        EncryptedTransactionModel::put_with_transaction_hash(
            &self.message.rollup_id,
            &transaction_hash,
            &self.message.encrypted_transaction,
        )?;

        EncryptedTransactionModel::put(
            &self.message.rollup_id,
            self.message.rollup_block_height,
            self.message.transaction_order,
            &self.message.encrypted_transaction,
        )?;

        self.message.order_commitment.put(
            &self.message.rollup_id,
            self.message.rollup_block_height,
            self.message.transaction_order,
        )?;

        rollup_metadata.current_gas += transaction_gas_limit;
        if rollup_metadata.transaction_order < self.message.transaction_order {
            rollup_metadata.transaction_order = self.message.transaction_order;
        }
        rollup_metadata.update()?;

        Ok(())
    }
}
