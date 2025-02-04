use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRawTransaction {
    pub message: SyncRawTransactionMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncRawTransactionMessage {
    pub rollup_id: String,
    pub rollup_block_height: u64,
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

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        // tracing::info!(
        //     "Sync raw transaction - rollup id: {:?}, rollup block height: {:?},
        // transaction order: {:?}, order commitment: {:?}",     self.message.
        // rollup_id,     self.message.rollup_block_height,
        //     self.message.transaction_order,
        //     self.message.order_commitment,
        // );

        let rollup = Rollup::get(&self.message.rollup_id)?;
        let mut rollup_metadata = RollupMetadata::get_mut(&self.message.rollup_id)?;
        let cluster = Cluster::get(
            rollup.platform,
            rollup.service_provider,
            &rollup.cluster_id,
            rollup_metadata.platform_block_height,
        )?;

        // Verify the leader signature
        let leader_address = cluster.get_leader_address(self.message.rollup_block_height)?;
        self.signature
            .verify_message(rollup.platform.into(), &self.message, leader_address)?;

        // Check the rollup block height
        if self.message.rollup_block_height != rollup_metadata.rollup_block_height {
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
            self.message.rollup_block_height,
            self.message.transaction_order,
            self.message.raw_transaction.clone(),
            self.message.is_direct_sent,
        )?;

        if let Some(order_commitment) = self.message.order_commitment {
            order_commitment.put(
                &self.message.rollup_id,
                self.message.rollup_block_height,
                self.message.transaction_order,
            )?;
        }

        rollup_metadata.transaction_order += 1;
        rollup_metadata.update()?;

        let merkle_tree = context
            .merkle_tree_manager()
            .get(&self.message.rollup_id)
            .await?;
        merkle_tree.add_data(transaction_hash.as_ref());

        Ok(())
    }
}
