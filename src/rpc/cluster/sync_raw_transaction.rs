use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRawTransaction {
    pub message: SyncRawTransactionMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRawTransactionMessage {
    pub rollup_id: String,
    pub batch_number: u64,
    pub transaction_order: u64,
    pub raw_transaction: RawTransaction,
    pub order_commitment: Option<OrderCommitment>,
    pub is_direct_sent: bool,
}

impl RpcParameter<AppState> for SyncRawTransaction {
    type Response = ();

    fn method() -> &'static str {
        "sync_raw_transaction"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        tracing::debug!(
            "Sync raw transaction - rollup id: {:?}, rollup block height: {:?},
        transaction order: {:?}, order commitment: {:?}",
            self.message.rollup_id,
            self.message.batch_number,
            self.message.transaction_order,
            self.message.order_commitment,
        );

        let transaction_gas_limit = self.message.raw_transaction.get_transaction_gas_limit()?;
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

        // Verify the leader signature
        let leader_tx_orderer_address = &cluster_metadata
            .leader_tx_orderer_rpc_info
            .unwrap()
            .tx_orderer_address;
        self.signature
            .verify_message(
                rollup.platform.into(),
                &self.message,
                leader_tx_orderer_address,
            )
            .map_err(|error| {
                tracing::error!("Failed to verify the leader signature: {:?}", error);
                Error::InvalidSignature
            })?;

        // Check the batch number
        if self.message.batch_number != rollup_metadata.batch_number {
            return Err(Error::BlockHeightMismatch.into());
        }

        let transaction_hash = self.message.raw_transaction.raw_transaction_hash();

        RawTransactionModel::put_with_transaction_hash(
            &self.message.rollup_id,
            &transaction_hash,
            self.message.raw_transaction.clone(),
            self.message.is_direct_sent,
        )?;

        RawTransactionModel::put(
            &self.message.rollup_id,
            self.message.batch_number,
            self.message.transaction_order,
            self.message.raw_transaction.clone(),
            self.message.is_direct_sent,
        )?;

        if let Some(order_commitment) = self.message.order_commitment {
            order_commitment.put(
                &self.message.rollup_id,
                self.message.batch_number,
                self.message.transaction_order,
            )?;
        }

        rollup_metadata.current_gas += transaction_gas_limit;
        if rollup_metadata.transaction_order < self.message.transaction_order {
            rollup_metadata.transaction_order = self.message.transaction_order;
        }
        rollup_metadata.update()?;

        Ok(())
    }
}
