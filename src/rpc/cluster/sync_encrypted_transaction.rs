use std::time::{SystemTime, UNIX_EPOCH};

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncEncryptedTransaction {
    pub rollup_id: RollupId,

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
        let start_sync_encrypted_transaction_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        let rollup_id = self.rollup_id.clone();
        let rollup = Rollup::get(&rollup_id)?;

        let cluster_metadata = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )
        .map_err(|error| {
            tracing::error!("Failed to get cluster metadata: {:?}", error);
            Error::ClusterMetadataNotFound
        })?;

        let cluster = Cluster::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
            cluster_metadata.platform_block_height,
        )?;

        // Verify the leader signature
        let is_valid_order_commitment = match self.order_commitment {
            OrderCommitment::Single(ref single_order_commitment) => match single_order_commitment {
                SingleOrderCommitment::Sign(sign_order_commitment) => {
                    let signer_address =
                        sign_order_commitment.get_signer_address(rollup.platform.into());

                    let tx_orderer_address_list = cluster.get_tx_orderer_address_list();

                    let leader_tx_orderer_address = tx_orderer_address_list
                        .iter()
                        .find(|&tx_orderer_address| signer_address == *tx_orderer_address);

                    leader_tx_orderer_address.is_some()
                }
                SingleOrderCommitment::TransactionHash(_) => true,
            },
            OrderCommitment::Bundle(_bundle) => {
                todo!("Handle bundle order commitment");
            }
        };

        if !is_valid_order_commitment {
            return Err(Error::InvalidOrderCommitment.into());
        }

        let transaction_hash = self.encrypted_transaction.raw_transaction_hash();

        EncryptedTransactionModel::put_with_transaction_hash(
            &rollup_id,
            &transaction_hash,
            &self.encrypted_transaction,
        )
        .map_err(|error| {
            tracing::error!("Failed to put encrypted transaction: {:?}", error);
            Error::Database(error)
        })?;

        EncryptedTransactionModel::put(
            &rollup_id,
            self.batch_number,
            self.transaction_order,
            &self.encrypted_transaction,
        )
        .map_err(|error| {
            tracing::error!("Failed to put encrypted transaction: {:?}", error);
            Error::Database(error)
        })?;

        self.order_commitment
            .put(&rollup_id, self.batch_number, self.transaction_order)
            .map_err(|error| {
                tracing::error!("Failed to put order commitment: {:?}", error);
                Error::Database(error)
            })?;

        let _ = context
            .decryptor()
            .add_encrypted_transaction_to_decrypt(
                rollup_id,
                self.batch_number,
                self.transaction_order,
                self.encrypted_transaction,
            )
            .await;

        let end_sync_encrypted_transaction_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        tracing::info!(
            "sync_encrypted_transaction - total take time: {:?}",
            end_sync_encrypted_transaction_time - start_sync_encrypted_transaction_time
        );

        Ok(())
    }
}
