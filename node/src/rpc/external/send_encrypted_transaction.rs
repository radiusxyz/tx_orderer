use crate::{
    rpc::{cluster::SyncEncryptedTransaction, prelude::*},
    types::*,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendEncryptedTransaction {
    pub rollup_id: RollupId,
    pub encrypted_transaction: EncryptedTransaction,
}

impl RpcParameter<AppState> for SendEncryptedTransaction {
    type Response = OrderCommitment;

    fn method() -> &'static str {
        "send_encrypted_transaction"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        // Use TransactionService to handle encrypted transaction processing
        let transaction_service = crate::services::TransactionService::new(context);
        let order_commitment = transaction_service
            .process_encrypted_transaction(&self.rollup_id, &self.encrypted_transaction)
            .await?;
        
        Ok(order_commitment)
    }
}


#[allow(clippy::too_many_arguments)]
pub fn sync_encrypted_transaction(
    context: AppState,
    platform: Platform,
    liveness_service_provider: LivenessServiceProvider,
    platform_block_height: u64,
    cluster_id: ClusterId,
    rollup_id: RollupId,
    batch_number: u64,
    transaction_order: u64,
    encrypted_transaction: EncryptedTransaction,
    order_commitment: OrderCommitment,
) {
    tokio::spawn(async move {
        let cluster = Cluster::get(
            platform,
            liveness_service_provider,
            &cluster_id,
            platform_block_height,
        )
        .expect("Failed to get cluster");

        let other_cluster_rpc_url_list = cluster.get_other_cluster_rpc_url_list();
        if other_cluster_rpc_url_list.is_empty() {
            return;
        }

        let sync_encypted_transaction = SyncEncryptedTransaction {
            rollup_id,
            batch_number,
            transaction_order,
            encrypted_transaction,
            order_commitment,
        };

        context
            .rpc_client()
            .fire_and_forget_multicast(
                other_cluster_rpc_url_list,
                SyncEncryptedTransaction::method(),
                &sync_encypted_transaction,
                Id::Null,
            )
            .await
    });
}

