mod skde_block_builder;

use radius_sdk::{
    json_rpc::{
        client::{Id, RpcClientError},
        server::RpcParameter,
    },
    signature::Signature,
};
use skde_block_builder::*;

use crate::{
    rpc::{
        cluster::{FinalizeBlockMessage, SyncBlock},
        external::{
            GetEncryptedTransactionWithOrderCommitment, GetRawTransactionWithOrderCommitment,
            GetRawTransactionWithOrderCommitmentResponse,
        },
    },
    state::AppState,
    types::*,
};

pub fn build_block(
    context: AppState,

    cluster: Cluster,

    finalize_block_message: FinalizeBlockMessage,
    rollup_signature: Signature,

    encrypted_transaction_type: EncryptedTransactionType,

    transaction_count: u64,
) {
    tracing::info!(
        "Build block - rollup id: {:?}, block number: {:?}, transaction count: {:?}",
        finalize_block_message.rollup_id,
        finalize_block_message.rollup_block_height,
        transaction_count
    );

    tokio::spawn(async move {
        let leader_sequencer_address = finalize_block_message.next_block_creator_address.clone();

        let block = match encrypted_transaction_type {
            EncryptedTransactionType::Pvde => unimplemented!(),
            EncryptedTransactionType::Skde => skde_build_block(
                context.clone(),
                &cluster,
                finalize_block_message.rollup_id.clone(),
                finalize_block_message.rollup_block_height.clone(),
                transaction_count,
                leader_sequencer_address,
                None,
            )
            .await
            .unwrap(),
            EncryptedTransactionType::NotSupport => unimplemented!(),
        };

        let _ = sync_block(
            context,
            cluster,
            finalize_block_message,
            rollup_signature,
            transaction_count,
            block.signature,
        )
        .await;
    });
}

pub async fn sync_block(
    context: AppState,
    cluster: Cluster,
    finalize_block_message: FinalizeBlockMessage,
    rollup_signature: Signature,
    transaction_count: u64,
    leader_sequencer_signature: Signature,
) {
    let parameter = SyncBlock {
        finalize_block_message,
        rollup_signature,
        transaction_count,
        leader_sequencer_signature,
    };

    let others_cluster_rpc_url_list = cluster.get_others_cluster_rpc_url_list();

    if others_cluster_rpc_url_list.is_empty() {
        tracing::info!("No other cluster RPC URLs available for synchronization");
        return;
    }

    match context
        .rpc_client()
        .multicast(
            others_cluster_rpc_url_list.clone(),
            SyncBlock::method(),
            &parameter,
            Id::Null,
        )
        .await
    {
        Ok(_) => tracing::info!(
            "Successfully synchronized block to {:?}",
            others_cluster_rpc_url_list
        ),
        Err(e) => tracing::error!("Failed to synchronize block: {:?}", e),
    }
}

pub fn follow_block(
    context: AppState,

    cluster: Cluster,

    finalize_block_message: FinalizeBlockMessage,
    encrypted_transaction_type: EncryptedTransactionType,

    transaction_count: u64,

    signature: Signature,
) {
    tracing::info!(
        "Follow building block - rollup id: {:?}, block number: {:?}, transaction count: {:?}",
        finalize_block_message.rollup_id,
        finalize_block_message.rollup_block_height,
        transaction_count
    );

    tokio::spawn(async move {
        match encrypted_transaction_type {
            EncryptedTransactionType::Pvde => unimplemented!(),
            EncryptedTransactionType::Skde => {
                skde_build_block(
                    context,
                    &cluster,
                    finalize_block_message.rollup_id.clone(),
                    finalize_block_message.rollup_block_height.clone(),
                    transaction_count,
                    finalize_block_message.next_block_creator_address.clone(),
                    Some(signature),
                )
                .await
            }
            EncryptedTransactionType::NotSupport => unimplemented!(),
        }
    });
}

pub fn get_encrypted_transaction_list(
    rollup_id: &str,
    rollup_block_height: u64,
    transaction_count: u64,
) -> Vec<Option<EncryptedTransaction>> {
    let mut encrypted_transaction_list =
        Vec::<Option<EncryptedTransaction>>::with_capacity(transaction_count as usize);

    for transaction_order in 0..transaction_count {
        let encrypted_transaction = match EncryptedTransactionModel::get(
            &rollup_id,
            rollup_block_height,
            transaction_order,
        ) {
            Ok(encrypted_transaction) => Some(encrypted_transaction),
            Err(error) => {
                if error.is_none_type() {
                    None
                } else {
                    panic!("block_builder: {:?}", error);
                }
            }
        };

        encrypted_transaction_list.push(encrypted_transaction.clone());
    }

    encrypted_transaction_list
}

pub fn get_raw_transaction_info_list(
    rollup_id: &str,
    rollup_block_height: u64,
    transaction_count: u64,
) -> Vec<Option<(RawTransaction, bool)>> {
    let mut raw_transaction_info_list =
        Vec::<Option<(RawTransaction, bool)>>::with_capacity(transaction_count as usize);

    for transaction_order in 0..transaction_count {
        let raw_transaction_info =
            match RawTransactionModel::get(&rollup_id, rollup_block_height, transaction_order) {
                Ok(raw_transaction_info) => Some(raw_transaction_info),
                Err(error) => {
                    if error.is_none_type() {
                        None
                    } else {
                        panic!("block_builder: {:?}", error);
                    }
                }
            };

        raw_transaction_info_list.push(raw_transaction_info.clone());
    }

    raw_transaction_info_list
}

async fn fetch_encrypted_transaction(
    context: AppState,
    cluster: &Cluster,
    rollup_id: &str,
    rollup_block_height: u64,
    transaction_order: u64,
) -> Result<EncryptedTransaction, RpcClientError> {
    let others_external_rpc_url_list = cluster.get_others_external_rpc_url_list();

    if others_external_rpc_url_list.is_empty() {
        tracing::warn!(
            rollup_id = %rollup_id,
            block_height = rollup_block_height,
            transaction_order = transaction_order,
            "No external RPC URLs available for fetching encrypted transactions."
        );
        return Err(RpcClientError::Response("NoEndpointsAvailable".to_string()));
    }

    let parameter = GetEncryptedTransactionWithOrderCommitment {
        rollup_id: rollup_id.to_owned(),
        rollup_block_height,
        transaction_order,
    };

    tracing::info!(
        rollup_id = %parameter.rollup_id,
        block_height = parameter.rollup_block_height,
        transaction_order = parameter.transaction_order,
        url_list = ?others_external_rpc_url_list,
        "Initiating fetch for encrypted transaction."
    );

    context
        .rpc_client()
        .fetch::<GetEncryptedTransactionWithOrderCommitment, EncryptedTransaction>(
            others_external_rpc_url_list,
            GetEncryptedTransactionWithOrderCommitment::method(),
            &parameter,
            Id::Null,
        )
        .await
        .map(|rpc_response| {
            tracing::info!(
                rollup_id = %parameter.rollup_id,
                block_height = parameter.rollup_block_height,
                transaction_order = parameter.transaction_order,
                "Successfully fetched encrypted transaction."
            );
            rpc_response
        })
        .map_err(|error| {
            tracing::error!(
                rollup_id = %parameter.rollup_id,
                block_height = parameter.rollup_block_height,
                transaction_order = parameter.transaction_order,
                error = ?error,
                "Failed to fetch encrypted transaction."
            );
            error
        })
}

async fn fetch_raw_transaction_info(
    context: AppState,
    cluster: &Cluster,
    rollup_id: &str,
    rollup_block_height: u64,
    transaction_order: u64,
) -> Result<(RawTransaction, bool), RpcClientError> {
    let others_external_rpc_url_list = cluster.get_others_external_rpc_url_list();

    if others_external_rpc_url_list.is_empty() {
        tracing::warn!(
            "No external RPC URLs available for fetching raw transactions. Rollup ID: {}, Block Height: {}, Order: {}",
            rollup_id, rollup_block_height, transaction_order
        );
        return Err(RpcClientError::Response("NoEndpointsAvailable".to_string()));
    }

    let parameter = GetRawTransactionWithOrderCommitment {
        rollup_id: rollup_id.to_owned(),
        rollup_block_height,
        transaction_order,
    };

    match context
        .rpc_client()
        .fetch::<GetRawTransactionWithOrderCommitment, GetRawTransactionWithOrderCommitmentResponse>(
            others_external_rpc_url_list,
            GetRawTransactionWithOrderCommitment::method(),
            &parameter,
            Id::Null,
        )
        .await
    {
        Ok(rpc_response) => {
            tracing::info!(
                "Successfully fetched raw transaction for Rollup ID: {}, Block Height: {}, Order: {}",
                parameter.rollup_id, parameter.rollup_block_height, parameter.transaction_order
            );
            Ok((rpc_response.raw_transaction, rpc_response.is_direct_sent))
        }
        Err(error) => {
            tracing::error!(
                "Failed to fetch raw transaction for Rollup ID: {}, Block Height: {}, Order: {}. Error: {:?}",
                parameter.rollup_id, parameter.rollup_block_height, parameter.transaction_order, error
            );
            Err(error)
        }
    }
}
