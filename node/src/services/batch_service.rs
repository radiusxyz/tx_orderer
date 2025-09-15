use tokio::time::{sleep, Duration};

use tx_orderer_primitives::{Error, Platform};
use crate::{
    state::AppState,
    types::*,
};

pub struct BatchService {
    app_state: AppState,
}

impl BatchService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Main service loop - runs indefinitely processing batch operations
    pub async fn run(&self) {
        tracing::info!("BatchService started");
        
        loop {
            // Service runs in background, processing batch events
            // Implementation will be added when extracting logic from batch finalization
            sleep(Duration::from_secs(1)).await;
        }
    }

    /// Process batch finalization
    pub async fn finalize_batch(
        &self,
        rollup_id: &RollupId,
        batch_number: u64,
    ) -> Result<(), Error> {
        let rollup = Rollup::get(rollup_id)?;
        let max_transaction_count_per_batch = rollup.max_transaction_count_per_batch;
        let cluster_meta = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;
        let cluster = Cluster::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
            cluster_meta.platform_block_height,
        )?;

        loop {
            tracing::info!("Finalizing batch - {}, {}", rollup_id, batch_number);

            let result = self.build_batch_data(
                &cluster,
                rollup_id,
                batch_number,
                max_transaction_count_per_batch,
            )
            .await;

            let BatchBuildResult {
                encrypted_transaction_list: encrypted_transactions,
                raw_transaction_list,
                batch_commitment,
            } = match result {
                Ok(data) => data,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            let signer = self.app_state.get_signer(rollup.platform).await?;
            let batch_creator_signature = signer.sign_message(&batch_commitment)?;

            let batch = Batch::new(
                batch_number,
                encrypted_transactions,
                raw_transaction_list,
                BatchCommitment::from(batch_commitment),
                signer.address().clone(),
                batch_creator_signature.clone(),
            );

            self.sync_batch_creation(
                cluster,
                rollup.platform,
                rollup_id.to_string(),
                batch_number,
                batch_commitment,
                batch_creator_signature,
            );

            CanProvideTransactionInfo::remove_can_provide_transaction_orders(rollup_id, batch_number)
                .expect("Failed to delete CanProvideTransactionInfo");

            Batch::put(&batch, rollup_id, batch_number)?;
            tracing::info!("Finalize batch DONE - {}, {}", rollup_id, batch_number);

            // Submit batch commitment to validation contract
            let validation_service = crate::services::ValidationService::new(self.app_state.clone());
            validation_service.submit_batch_commitment(&rollup, batch_number, &batch_commitment).await.ok();

            break;
        }

        Ok(())
    }

    /// Create batch from leader signature
    pub async fn create_batch(
        &self,
        rollup_id: &RollupId,
        batch_number: u64,
        batch_creator_signature: radius_sdk::signature::Signature,
        leader_tx_orderer_signature: radius_sdk::signature::Signature,
    ) -> Result<(), Error> {
        let rollup = Rollup::get(rollup_id)?;
        let max_transaction_count_per_batch = rollup.max_transaction_count_per_batch;
        let cluster_meta = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;
        let cluster = Cluster::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
            cluster_meta.platform_block_height,
        )?;

        loop {
            tracing::info!("Creating batch - {}, {}", rollup_id, batch_number);

            let result = self.build_batch_data(
                &cluster,
                rollup_id,
                batch_number,
                max_transaction_count_per_batch,
            )
            .await;

            let BatchBuildResult {
                encrypted_transaction_list: encrypted_transactions,
                raw_transaction_list: raw_transactions,
                batch_commitment,
            } = match result {
                Ok(data) => data,
                Err(error) => {
                    tracing::error!(
                        "Failed to build batch data - rollup_id: {:?}, batch_number: {:?} - error: {:?}",
                        rollup_id,
                        batch_number,
                        error
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            let batch_creation_message = crate::rpc::BatchCreationMessage {
                rollup_id: rollup_id.to_string(),
                batch_number,
                batch_commitment,
                batch_creator_signature: batch_creator_signature.clone(),
            };

            if let Ok(signer_address) = leader_tx_orderer_signature
                .get_signer_address(rollup.platform.into(), &batch_creation_message)
            {
                let tx_orderer_address_list = cluster.get_tx_orderer_address_list();

                if let Some(leader_tx_orderer_address) = tx_orderer_address_list
                    .iter()
                    .find(|&tx_orderer_address| signer_address == *tx_orderer_address)
                {
                    let batch = Batch::new(
                        batch_number,
                        encrypted_transactions,
                        raw_transactions,
                        BatchCommitment::from(batch_commitment),
                        leader_tx_orderer_address.clone(),
                        batch_creator_signature,
                    );

                    CanProvideTransactionInfo::remove_can_provide_transaction_orders(
                        rollup_id,
                        batch_number,
                    )
                    .expect("Failed to delete CanProvideTransactionInfo");

                    Batch::put(&batch, rollup_id, batch_number)?;
                } else {
                    tracing::error!(
                        "Failed to verify leader tx orderer signature - rollup_id: {:?}, batch_number: {:?} / tx_orderer_address_list: {:?} / signer_address: {:?} / batch_commitment: {:?} / raw_transaction_list_count: {:?}",
                        rollup_id,
                        batch_number,
                        tx_orderer_address_list,
                        signer_address,
                        BatchCommitment::from(batch_commitment),
                        raw_transactions.len()
                    );

                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            } else {
                tracing::error!(
                    "Failed to verify leader tx orderer signature (2) - rollup_id: {:?}, batch_number: {:?} / batch_creation_message: {:?}",
                    rollup_id,
                    batch_number,
                    batch_creation_message
                );
            }

            break;
        }

        Ok(())
    }

    /// Build batch data with encrypted and raw transactions
    pub async fn build_batch_data(
        &self,
        cluster: &Cluster,
        rollup_id: &RollupId,
        batch_number: u64,
        max_transaction_count_per_batch: u64,
    ) -> Result<BatchBuildResult, Error> {
        let rpc_client = self.app_state.rpc_client();

        let mut encrypted_transaction_list =
            self.get_encrypted_transaction_list(rollup_id, batch_number, max_transaction_count_per_batch);

        let raw_transaction_info_list = self.get_raw_transaction_info_list(
            rollup_id,
            rpc_client,
            cluster,
            batch_number,
            max_transaction_count_per_batch,
        )
        .await?;

        for transaction_order in 0..encrypted_transaction_list.len() {
            let encrypted_transaction = &encrypted_transaction_list[transaction_order];
            let (_, is_direct_sent) = &raw_transaction_info_list[transaction_order];

            if encrypted_transaction.is_none() && !is_direct_sent {
                let encrypted_transaction = crate::utils::fetch_encrypted_transaction(
                    rpc_client,
                    cluster,
                    rollup_id,
                    batch_number,
                    transaction_order as u64,
                )
                .await?;

                encrypted_transaction_list[transaction_order] = Some(encrypted_transaction);
            }
        }

        let merkle_tree = tx_orderer_primitives::MerkleTree::new();
        for (raw_transaction, _) in &raw_transaction_info_list {
            merkle_tree
                .add_data(raw_transaction.raw_transaction_hash().as_ref())
                .await;
        }
        merkle_tree.finalize_tree().await;
        let batch_commitment = merkle_tree.get_merkle_root().await;

        let raw_transaction_list: Vec<RawTransaction> = raw_transaction_info_list
            .into_iter()
            .map(|(raw_transaction, _)| raw_transaction)
            .collect();

        Ok(BatchBuildResult {
            encrypted_transaction_list,
            raw_transaction_list,
            batch_commitment,
        })
    }

    /// Get encrypted transaction list for a batch
    pub fn get_encrypted_transaction_list(
        &self,
        rollup_id: &RollupId,
        rollup_batch_number: u64,
        transaction_count: u64,
    ) -> Vec<Option<EncryptedTransaction>> {
        let mut encrypted_transaction_list =
            Vec::<Option<EncryptedTransaction>>::with_capacity(transaction_count as usize);

        for transaction_order in 0..transaction_count {
            let encrypted_transaction = match EncryptedTransactionModel::get(
                rollup_id,
                rollup_batch_number,
                transaction_order,
            ) {
                Ok(encrypted_transaction) => Some(encrypted_transaction),
                Err(error) => {
                    if error.is_none_type() {
                        None
                    } else {
                        panic!("batch_builder: {:?}", error);
                    }
                }
            };

            encrypted_transaction_list.push(encrypted_transaction);
        }

        encrypted_transaction_list
    }

    /// Get raw transaction info list for a batch
    pub async fn get_raw_transaction_info_list(
        &self,
        rollup_id: &RollupId,
        rpc_client: &radius_sdk::json_rpc::client::RpcClient,
        cluster: &Cluster,
        batch_number: u64,
        max_transaction_count_per_batch: u64,
    ) -> Result<Vec<(RawTransaction, bool)>, Error> {
        let mut raw_transaction_info_list =
            Vec::<(RawTransaction, bool)>::with_capacity(max_transaction_count_per_batch as usize);

        for transaction_order in 0..max_transaction_count_per_batch {
            let raw_transaction_info =
                match RawTransactionModel::get(rollup_id, batch_number, transaction_order) {
                    Ok(raw_transaction_info) => raw_transaction_info,
                    Err(_error) => {
                        let raw_transaction_info = crate::utils::fetch_raw_transaction_info(
                            rpc_client,
                            cluster,
                            rollup_id,
                            batch_number,
                            transaction_order,
                        )
                        .await?;

                        raw_transaction_info
                    }
                };

            raw_transaction_info_list.push(raw_transaction_info);
        }

        tracing::info!(
            "get_raw_transaction_info_list - rollup_id: {:?} / batch_number: {:?} / max_transaction_count_per_batch: {:?} / raw_transaction_info_list_count: {:?}",
            rollup_id,
            batch_number,
            max_transaction_count_per_batch,
            raw_transaction_info_list.len()
        );
        Ok(raw_transaction_info_list)
    }

    /// Sync batch creation to other nodes
    pub fn sync_batch_creation(
        &self,
        cluster: Cluster,
        _platform: Platform,
        rollup_id: String,
        batch_number: u64,
        batch_commitment: [u8; 32],
        batch_creator_signature: radius_sdk::signature::Signature,
    ) {
        let context = self.app_state.clone();
        tokio::spawn(async move {
            let other_cluster_rpc_url_list = cluster.get_other_cluster_rpc_url_list();
            if other_cluster_rpc_url_list.is_empty() {
                return;
            }

            let batch_creation_message = crate::rpc::BatchCreationMessage {
                rollup_id,
                batch_number,
                batch_commitment,
                batch_creator_signature,
            };

            context
                .rpc_client()
                .fire_and_forget_multicast(
                    other_cluster_rpc_url_list,
                    "sync_batch_creation",
                    &batch_creation_message,
                    radius_sdk::json_rpc::client::Id::Null,
                )
                .await
        });
    }
}

/// Result of building batch data
pub struct BatchBuildResult {
    pub encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
    pub raw_transaction_list: Vec<RawTransaction>,
    pub batch_commitment: [u8; 32],
}