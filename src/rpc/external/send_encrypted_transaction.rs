use crate::{
    rpc::{cluster::SyncEncryptedTransaction, prelude::*},
    task::finalize_batch,
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
        let rollup = Rollup::get(&self.rollup_id)?;

        // 1. Check supported encrypted transaction
        check_supported_encrypted_transaction(&rollup, &self.encrypted_transaction)?;

        let mut mut_rollup_metadata =
            RollupMetadata::get_mut(&self.rollup_id).map_err(|error| {
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
            let transaction_hash = self.encrypted_transaction.raw_transaction_hash();

            mut_rollup_metadata.transaction_order += 1;

            let is_updated = mut_rollup_metadata.check_and_update_batch_info();

            mut_rollup_metadata.update()?;

            if is_updated {
                context
                    .merkle_tree_manager()
                    .insert(&self.rollup_id, MerkleTree::new())
                    .await;

                finalize_batch(context.clone(), &self.rollup_id, batch_number);
            }

            EncryptedTransactionModel::put_with_transaction_hash(
                &self.rollup_id,
                &transaction_hash,
                &self.encrypted_transaction,
            )?;

            EncryptedTransactionModel::put(
                &self.rollup_id,
                batch_number,
                transaction_order,
                &self.encrypted_transaction,
            )?;

            let merkle_tree = context.merkle_tree_manager().get(&self.rollup_id).await?;
            let (_, pre_merkle_path) = merkle_tree.add_data(transaction_hash.as_ref()).await;

            let order_commitment = issue_order_commitment(
                context.clone(),
                rollup.platform,
                self.rollup_id.clone(),
                rollup.order_commitment_type,
                transaction_hash,
                batch_number,
                transaction_order,
                pre_merkle_path,
            )
            .await?;
            order_commitment.put(&self.rollup_id, batch_number, transaction_order)?;

            sync_encrypted_transaction(
                context.clone(),
                rollup.platform,
                rollup.liveness_service_provider,
                cluster_metadata.platform_block_height,
                rollup.cluster_id.clone(),
                self.rollup_id.clone(),
                batch_number,
                transaction_order,
                self.encrypted_transaction.clone(),
                order_commitment.clone(),
            );

            let _ = context
                .decryptor()
                .add_encrypted_transaction_to_decrypt(
                    self.rollup_id,
                    batch_number,
                    transaction_order,
                    self.encrypted_transaction,
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

                    match context
                        .rpc_client()
                        .request(
                            leader_external_rpc_url,
                            SendEncryptedTransaction::method(),
                            &self,
                            Id::Null,
                        )
                        .await
                    {
                        Ok(response) => Ok(response),
                        Err(error) => Err(error.into()),
                    }
                }
                None => {
                    return Err(Error::EmptyLeader)?;
                }
            }
        }
    }
}

fn check_supported_encrypted_transaction(
    rollup: &Rollup,
    encrypted_transaction: &EncryptedTransaction,
) -> Result<(), Error> {
    match rollup.encrypted_transaction_type {
        EncryptedTransactionType::Pvde => {}
        EncryptedTransactionType::Skde => {
            if !matches!(encrypted_transaction, EncryptedTransaction::Skde(_)) {
                return Err(Error::UnsupportedEncryptedMempool);
            }
        }
        EncryptedTransactionType::NotSupport => return Err(Error::UnsupportedEncryptedMempool),
    };

    Ok(())
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

#[allow(clippy::too_many_arguments)]
pub async fn issue_order_commitment(
    context: AppState,
    platform: Platform,
    rollup_id: RollupId,
    order_commitment_type: OrderCommitmentType,
    transaction_hash: RawTransactionHash,
    batch_number: u64,
    transaction_order: u64,
    pre_merkle_path: Vec<[u8; 32]>,
) -> Result<OrderCommitment, RpcError> {
    match order_commitment_type {
        OrderCommitmentType::TransactionHash => Ok(OrderCommitment::Single(
            SingleOrderCommitment::TransactionHash(TransactionHashOrderCommitment::new(
                transaction_hash.as_string(),
            )),
        )),
        OrderCommitmentType::Sign => {
            let signer = context.get_signer(platform).await?;
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
