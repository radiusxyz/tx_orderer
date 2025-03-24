use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncEncryptedTransaction {
    pub message: SyncEncryptedTransactionMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncEncryptedTransactionMessage {
    pub rollup_id: String,
    pub batch_number: u64,
    pub transaction_order: u64,
    pub encrypted_transaction: EncryptedTransaction,
    pub order_commitment: OrderCommitment,
}

impl RpcParameter<AppState> for SyncEncryptedTransaction {
    type Response = ();

    fn method() -> &'static str {
        "sync_encrypted_transaction"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        tracing::debug!(
            "Sync encrypted transaction - rollup id: {:?}, rollup block height: {:?}, transaction order: {:?}, order commitment: {:?}",
            self.message.rollup_id,
            self.message.batch_number,
            self.message.transaction_order,
            self.message.order_commitment,
        );

        let transaction_gas_limit = self
            .message
            .encrypted_transaction
            .get_transaction_gas_limit()?;

        let rollup = Rollup::get(&self.message.rollup_id)?;
        let cluster_metadata = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;
        let mut rollup_metadata = RollupMetadata::get_mut(&self.message.rollup_id)?;

        if cluster_metadata.leader_tx_orderer_rpc_info.is_none() {
            return Err(Error::EmptyLeader.into());
        }

        if cluster_metadata.leader_tx_orderer_rpc_info.is_none() {
            return Err(Error::EmptyLeader.into());
        }
        // Verify the leader signature
        let leader_tx_orderer_address = &cluster_metadata
            .leader_tx_orderer_rpc_info
            .unwrap()
            .tx_orderer_address;
        self.signature.verify_message(
            rollup.platform.into(),
            &self.message,
            leader_tx_orderer_address,
        )?;

        // Check the rollup block height
        if self.message.batch_number != rollup_metadata.batch_number {
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
            self.message.batch_number,
            self.message.transaction_order,
            &self.message.encrypted_transaction,
        )?;

        self.message.order_commitment.put(
            &self.message.rollup_id,
            self.message.batch_number,
            self.message.transaction_order,
        )?;

        rollup_metadata.current_gas += transaction_gas_limit;
        if rollup_metadata.transaction_order < self.message.transaction_order {
            rollup_metadata.transaction_order = self.message.transaction_order;
        }
        rollup_metadata.update()?;

        let _ = context
            .decryptor()
            .add_encrypted_transaction_to_decrypt(
                self.message.rollup_id,
                self.message.batch_number,
                self.message.transaction_order,
                self.message.encrypted_transaction,
            )
            .await;

        Ok(())
    }
}
