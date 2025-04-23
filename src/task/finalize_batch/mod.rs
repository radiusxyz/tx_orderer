// mod skde_batch_builder;
mod validation;

use radius_sdk::{json_rpc::client::RpcClient, signature::Signature};
use tokio::time::Duration;
use validation::submit_batch_commitment;

use crate::{
    error::Error,
    rpc::{cluster::BatchCreationMessage, external::sync_batch_creation},
    state::AppState,
    types::*,
    util::{fetch_encrypted_transaction, fetch_raw_transaction_info},
};

pub fn finalize_batch(context: AppState, rollup_id: &RollupId, batch_number: u64) {
    if Batch::get(rollup_id, batch_number).is_ok() {
        tracing::info!(
            "Finalize batch - rollup id: {:?}, batch number: {:?} already exists",
            rollup_id,
            batch_number
        );
        return;
    }

    let rollup_id = rollup_id.to_string();
    tokio::spawn(async move {
        if let Err(error) = finalize_batch_task(context, &rollup_id, batch_number).await {
            tracing::error!(
                "Failed to finalize batch - rollup_id: {:?}, batch_number: {:?}, error: {:?}",
                rollup_id,
                batch_number,
                error
            );
        }
    });
}

async fn finalize_batch_task(
    context: AppState,
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

        let result = build_batch_data(
            &context,
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

        let signer = context.get_signer(rollup.platform).await?;
        let batch_creator_signature = signer.sign_message(&batch_commitment)?;

        let batch = Batch::new(
            batch_number,
            encrypted_transactions,
            raw_transaction_list,
            BatchCommitment::from(batch_commitment),
            signer.address().clone(),
            batch_creator_signature.clone(),
        );

        sync_batch_creation(
            context.clone(),
            cluster,
            rollup.platform,
            rollup_id.to_string(),
            batch_number,
            batch_commitment,
            batch_creator_signature,
        );

        CanProvideTransactionInfo::remove_can_provide_transaction_orders(&rollup_id, batch_number)
            .expect("Failed to delete CanProvideTransactionInfo");

        Batch::put(&batch, rollup_id, batch_number)?;
        tracing::info!("Finalize batch DONE - {}, {}", rollup_id, batch_number);

        submit_batch_commitment(context, &rollup, batch_number, &batch_commitment).await;

        break;
    }

    Ok(())
}

pub fn create_batch(
    context: AppState,
    rollup_id: &RollupId,
    batch_number: u64,
    batch_creator_signature: Signature,
    leader_tx_orderer_signature: Signature,
) {
    if Batch::get(rollup_id, batch_number).is_ok() {
        tracing::info!(
            "Finalize batch - rollup id: {:?}, batch number: {:?} already exists",
            rollup_id,
            batch_number
        );
        return;
    }

    let rollup_id = rollup_id.to_string();
    tokio::spawn(async move {
        if let Err(error) = create_batch_task(
            context,
            &rollup_id,
            batch_number,
            batch_creator_signature,
            leader_tx_orderer_signature,
        )
        .await
        {
            tracing::error!(
                "Failed to create batch - rollup_id: {:?}, batch_number: {:?}, error: {:?}",
                rollup_id,
                batch_number,
                error
            );
        }
    });
}

pub async fn create_batch_task(
    context: AppState,
    rollup_id: &RollupId,
    batch_number: u64,
    batch_creator_signature: Signature,
    leader_tx_orderer_signature: Signature,
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

        let result = build_batch_data(
            &context,
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

        let batch_creation_massage = BatchCreationMessage {
            rollup_id: rollup_id.to_string(),
            batch_number,
            batch_commitment,
            batch_creator_signature: batch_creator_signature.clone(),
        };

        if let Ok(signer_address) = leader_tx_orderer_signature
            .get_signer_address(rollup.platform.into(), &batch_creation_massage)
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
                    &rollup_id,
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
                "Failed to verify leader tx orderer signature (2) - rollup_id: {:?}, batch_number: {:?} / batch_creation_massage: {:?}",
                rollup_id,
                batch_number,
                batch_creation_massage
            );
        }

        break;
    }

    Ok(())
}

struct BatchBuildResult {
    encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
    raw_transaction_list: Vec<RawTransaction>,
    batch_commitment: [u8; 32],
}

async fn build_batch_data(
    context: &AppState,

    cluster: &Cluster,
    rollup_id: &RollupId,
    batch_number: u64,
    max_transaction_count_per_batch: u64,
) -> Result<BatchBuildResult, Error> {
    let rpc_client = context.rpc_client();

    let mut encrypted_transaction_list =
        get_encrypted_transaction_list(rollup_id, batch_number, max_transaction_count_per_batch);

    let raw_transaction_info_list = get_raw_transaction_info_list(
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
            let encrypted_transaction = fetch_encrypted_transaction(
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

    let merkle_tree = MerkleTree::new();
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

pub fn get_encrypted_transaction_list(
    rollup_id: &RollupId,
    rollup_batch_number: u64,
    transaction_count: u64,
) -> Vec<Option<EncryptedTransaction>> {
    let mut encrypted_transaction_list =
        Vec::<Option<EncryptedTransaction>>::with_capacity(transaction_count as usize);

    for transaction_order in 0..transaction_count {
        let encrypted_transaction = match EncryptedTransactionModel::get(
            &rollup_id,
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

pub async fn get_raw_transaction_info_list(
    rollup_id: &RollupId,
    rpc_client: &RpcClient,
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
                    let raw_transaction_info = fetch_raw_transaction_info(
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
