use tokio::time::{sleep, Duration};

use tx_orderer_primitives::{Error, MerkleTree, Platform, LivenessServiceProvider};
use crate::{
    state::AppState,
    types::*,
    tasks::finalize_batch,
    rpc::{SyncEncryptedTransaction, SyncRawTransaction},
};

pub struct TransactionService {
    app_state: AppState,
}

impl TransactionService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Main service loop - runs indefinitely processing transactions
    pub async fn run(&self) {
        tracing::info!("TransactionService started");
        
        loop {
            // Service runs in background, processing events
            // Implementation will be added when extracting logic from RPC handlers
            sleep(Duration::from_secs(1)).await;
        }
    }

    /// Process encrypted transaction submission
    pub async fn process_encrypted_transaction(
        &self,
        rollup_id: &RollupId,
        encrypted_transaction: &EncryptedTransaction,
    ) -> Result<OrderCommitment, Error> {
        let rollup = Rollup::get(rollup_id)?;

        // 1. Check supported encrypted transaction
        self.check_supported_encrypted_transaction(&rollup, encrypted_transaction)?;

        let mut mut_rollup_metadata =
            RollupMetadata::get_mut(rollup_id).map_err(|error| {
                tracing::error!("Failed to get rollup metadata: {:?}", error);
                Error::RollupMetadataNotFound
            })?;

        // 2. Check is leader
        let cluster_metadata = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;

        if cluster_metadata.is_leader {
            let batch_number = mut_rollup_metadata.batch_number;
            let transaction_order = mut_rollup_metadata.transaction_order;
            let transaction_hash = encrypted_transaction.raw_transaction_hash();

            mut_rollup_metadata.transaction_order += 1;

            let is_updated = mut_rollup_metadata.check_and_update_batch_info();

            mut_rollup_metadata.update()?;

            if is_updated {
                self.app_state
                    .merkle_tree_manager()
                    .insert(rollup_id, MerkleTree::new())
                    .await;

                finalize_batch(self.app_state.clone(), rollup_id, batch_number);
            }

            EncryptedTransactionModel::put_with_transaction_hash(
                rollup_id,
                &transaction_hash,
                encrypted_transaction,
            )?;

            EncryptedTransactionModel::put(
                rollup_id,
                batch_number,
                transaction_order,
                encrypted_transaction,
            )?;

            let merkle_tree = self.app_state.merkle_tree_manager().get(rollup_id).await?;
            let (_, pre_merkle_path) = merkle_tree.add_data(transaction_hash.as_ref()).await;

            let order_commitment = self.issue_order_commitment(
                rollup.platform,
                rollup_id.clone(),
                rollup.order_commitment_type,
                transaction_hash,
                batch_number,
                transaction_order,
                pre_merkle_path,
            ).await?;
            order_commitment.put(rollup_id, batch_number, transaction_order)?;

            self.sync_encrypted_transaction(
                rollup.platform,
                rollup.liveness_service_provider,
                cluster_metadata.platform_block_height,
                rollup.cluster_id.clone(),
                rollup_id.clone(),
                batch_number,
                transaction_order,
                encrypted_transaction.clone(),
                order_commitment.clone(),
            );

            let _ = self.app_state
                .decryptor()
                .add_encrypted_transaction_to_decrypt(
                    rollup_id.clone(),
                    batch_number,
                    transaction_order,
                    encrypted_transaction.clone(),
                )
                .await;

            Ok(order_commitment)
        } else {
            drop(mut_rollup_metadata);

            match cluster_metadata.leader_tx_orderer_rpc_info {
                Some(leader_tx_orderer_rpc_info) => {
                    let leader_external_rpc_url = leader_tx_orderer_rpc_info
                        .external_rpc_url
                        .clone()
                        .ok_or(Error::EmptyLeaderClusterRpcUrl)?;

                    match self.app_state
                        .rpc_client()
                        .request(
                            leader_external_rpc_url,
                            "send_encrypted_transaction",
                            &serde_json::json!({
                                "rollup_id": rollup_id,
                                "encrypted_transaction": encrypted_transaction
                            }),
                            radius_sdk::json_rpc::client::Id::Null,
                        )
                        .await
                    {
                        Ok(response) => Ok(response),
                        Err(error) => Err(error.into()),
                    }
                }
                None => {
                    Err(Error::EmptyLeader)
                }
            }
        }
    }

    /// Process raw transaction submission
    pub async fn process_raw_transaction(
        &self,
        rollup_id: &RollupId,
        raw_transaction: &RawTransaction,
    ) -> Result<OrderCommitment, Error> {
        let rollup = Rollup::get(rollup_id)?;

        let mut mut_rollup_metadata = RollupMetadata::get_mut(rollup_id)?;

        let cluster_metadata = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )
        .map_err(|error| {
            tracing::error!("Failed to get cluster metadata: {:?}", error);
            Error::ClusterMetadataNotFound
        })?;

        if cluster_metadata.is_leader {
            let cluster = Cluster::get(
                rollup.platform,
                rollup.liveness_service_provider,
                &rollup.cluster_id,
                cluster_metadata.platform_block_height,
            )
            .map_err(|error| {
                tracing::error!("Failed to get cluster: {:?}", error);
                Error::ClusterNotFound
            })?;

            let batch_number = mut_rollup_metadata.batch_number;
            let transaction_order = mut_rollup_metadata.transaction_order;
            let transaction_hash = raw_transaction.raw_transaction_hash();

            RawTransactionModel::put_with_transaction_hash(
                rollup_id,
                &transaction_hash,
                raw_transaction.clone(),
                true,
            )?;

            RawTransactionModel::put(
                rollup_id,
                batch_number,
                transaction_order,
                raw_transaction.clone(),
                true,
            )?;

            let merkle_tree = self.app_state.merkle_tree_manager().get(rollup_id).await?;
            let (_, pre_merkle_path) = merkle_tree.add_data(transaction_hash.as_ref()).await;
            drop(merkle_tree);

            mut_rollup_metadata.transaction_order += 1;
            CanProvideTransactionInfo::add_can_provide_transaction_orders(
                rollup_id,
                batch_number,
                vec![transaction_order],
            )?;

            let is_updated = mut_rollup_metadata.check_and_update_batch_info();

            mut_rollup_metadata.update()?;

            if is_updated {
                self.app_state
                    .merkle_tree_manager()
                    .insert(rollup_id, MerkleTree::new())
                    .await;

                finalize_batch(self.app_state.clone(), rollup_id, batch_number);
            }

            let order_commitment = self.issue_order_commitment(
                rollup.platform,
                rollup_id.clone(),
                rollup.order_commitment_type,
                transaction_hash.clone(),
                batch_number,
                transaction_order,
                pre_merkle_path,
            )
            .await?;

            order_commitment.put(rollup_id, batch_number, transaction_order)?;

            self.sync_raw_transaction(
                cluster,
                rollup_id.clone(),
                batch_number,
                transaction_order,
                raw_transaction.clone(),
                order_commitment.clone(),
                true,
            );

            let builder_rpc_url = self.app_state.config().builder_rpc_url.clone();
            let cloned_rpc_client = self.app_state.rpc_client();

            if builder_rpc_url.is_some() {
                match raw_transaction {
                    RawTransaction::Eth(eth_raw_transaction) => {
                        let params = serde_json::json!([
                            eth_raw_transaction.0,
                            batch_number,
                            transaction_order
                        ]);

                        let _transaction_hash: String = cloned_rpc_client
                            .request(
                                &builder_rpc_url.unwrap(),
                                "eth_sendRawTransaction",
                                &params,
                                radius_sdk::json_rpc::client::Id::Null,
                            )
                            .await
                            .map_err(|error| {
                                tracing::error!("Failed to send raw transaction: {:?}", error);
                                Error::RpcClient(error)
                            })?;
                    }
                    RawTransaction::EthBundle(_eth_bundle_raw_transaction) => {
                        unimplemented!("EthBundle raw transaction is not supported yet");
                    }
                }
            }

            match rollup.order_commitment_type {
                OrderCommitmentType::TransactionHash => Ok(OrderCommitment::Single(
                    SingleOrderCommitment::TransactionHash(TransactionHashOrderCommitment::new(
                        transaction_hash.as_string(),
                    )),
                )),
                OrderCommitmentType::Sign => Ok(order_commitment),
            }
        } else {
            drop(mut_rollup_metadata);

            match cluster_metadata.leader_tx_orderer_rpc_info {
                Some(leader_tx_orderer_rpc_info) => {
                    let leader_external_rpc_url = leader_tx_orderer_rpc_info
                        .external_rpc_url
                        .clone()
                        .ok_or(Error::EmptyLeaderClusterRpcUrl)?;

                    match self.app_state
                        .rpc_client()
                        .request(
                            leader_external_rpc_url,
                            "send_raw_transaction",
                            &serde_json::json!({
                                "rollup_id": rollup_id,
                                "raw_transaction": raw_transaction
                            }),
                            radius_sdk::json_rpc::client::Id::Null,
                        )
                        .await
                    {
                        Ok(response) => Ok(response),
                        Err(error) => {
                            tracing::error!(
                                "Send raw transaction - leader external rpc error: {:?}",
                                error
                            );
                            Err(error.into())
                        }
                    }
                }
                None => {
                    tracing::error!("Send raw transaction - leader tx orderer rpc info is None");
                    Err(Error::EmptyLeader)
                }
            }
        }
    }

    /// Check if encrypted transaction is supported for the rollup
    pub fn check_supported_encrypted_transaction(
        &self,
        rollup: &Rollup,
        encrypted_transaction: &EncryptedTransaction,
    ) -> Result<(), Error> {
        match rollup.encrypted_transaction_type {
            EncryptedTransactionType::Pvde => {},
            EncryptedTransactionType::Skde => {
                if !matches!(encrypted_transaction, EncryptedTransaction::Skde(_)) {
                    return Err(Error::UnsupportedEncryptedMempool);
                }
            }
            EncryptedTransactionType::NotSupport => return Err(Error::UnsupportedEncryptedMempool),
        };

        Ok(())
    }

    /// Update transaction order and batch info
    pub async fn update_transaction_order(
        &self,
        rollup_id: &RollupId,
    ) -> Result<(u64, u64), Error> {
        let mut mut_rollup_metadata = RollupMetadata::get_mut(rollup_id)
            .map_err(|error| {
                tracing::error!("Failed to get rollup metadata: {:?}", error);
                Error::RollupMetadataNotFound
            })?;

        let batch_number = mut_rollup_metadata.batch_number;
        let transaction_order = mut_rollup_metadata.transaction_order;

        mut_rollup_metadata.transaction_order += 1;
        let is_updated = mut_rollup_metadata.check_and_update_batch_info();
        mut_rollup_metadata.update()?;

        if is_updated {
            self.app_state
                .merkle_tree_manager()
                .insert(rollup_id, MerkleTree::new())
                .await;

            finalize_batch(self.app_state.clone(), rollup_id, batch_number);
        }

        Ok((batch_number, transaction_order))
    }

    /// Issue order commitment for a transaction
    pub async fn issue_order_commitment(
        &self,
        platform: Platform,
        rollup_id: RollupId,
        order_commitment_type: OrderCommitmentType,
        transaction_hash: RawTransactionHash,
        batch_number: u64,
        transaction_order: u64,
        pre_merkle_path: Vec<[u8; 32]>,
    ) -> Result<OrderCommitment, Error> {
        match order_commitment_type {
            OrderCommitmentType::TransactionHash => Ok(OrderCommitment::Single(
                SingleOrderCommitment::TransactionHash(TransactionHashOrderCommitment::new(
                    transaction_hash.as_string(),
                )),
            )),
            OrderCommitmentType::Sign => {
                let signer = self.app_state.get_signer(platform).await?;
                let order_commitment_data = OrderCommitmentData {
                    rollup_id,
                    batch_number,
                    transaction_hash: transaction_hash.as_string(),
                    transaction_order,
                    pre_merkle_path,
                };
                let order_commitment = SignOrderCommitment {
                    data: order_commitment_data.clone(),
                    signature: signer.sign_message(&order_commitment_data)?,
                };

                Ok(OrderCommitment::Single(SingleOrderCommitment::Sign(
                    order_commitment,
                )))
            }
        }
    }

    /// Sync encrypted transaction to other cluster nodes
    #[allow(clippy::too_many_arguments)]
    pub fn sync_encrypted_transaction(
        &self,
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
        let context = self.app_state.clone();
        tokio::spawn(async move {
            let cluster = match Cluster::get(
                platform,
                liveness_service_provider,
                &cluster_id,
                platform_block_height,
            ) {
                Ok(cluster) => cluster,
                Err(_) => {
                    tracing::error!("Failed to get cluster");
                    return;
                }
            };

            let other_cluster_rpc_url_list = cluster.get_other_cluster_rpc_url_list();
            if other_cluster_rpc_url_list.is_empty() {
                return;
            }

            let sync_encrypted_transaction = SyncEncryptedTransaction {
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
                    "sync_encrypted_transaction",
                    &sync_encrypted_transaction,
                    radius_sdk::json_rpc::client::Id::Null,
                )
                .await
        });
    }

    /// Sync raw transaction to other cluster nodes
    #[allow(clippy::too_many_arguments)]
    pub fn sync_raw_transaction(
        &self,
        cluster: Cluster,
        rollup_id: RollupId,
        batch_number: u64,
        transaction_order: u64,
        raw_transaction: RawTransaction,
        order_commitment: OrderCommitment,
        is_direct_sent: bool,
    ) {
        let context = self.app_state.clone();
        tokio::spawn(async move {
            let other_cluster_rpc_url_list = cluster.get_other_cluster_rpc_url_list();
            if other_cluster_rpc_url_list.is_empty() {
                return;
            }

            let sync_raw_transaction = SyncRawTransaction {
                rollup_id,
                batch_number,
                transaction_order,
                raw_transaction,
                order_commitment,
                is_direct_sent,
            };

            context
                .rpc_client()
                .fire_and_forget_multicast(
                    other_cluster_rpc_url_list,
                    "sync_raw_transaction",
                    &sync_raw_transaction,
                    radius_sdk::json_rpc::client::Id::Null,
                )
                .await
        });
    }
}