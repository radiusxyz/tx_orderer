use std::collections::HashMap;

use radius_sdk::signature::{Address, Signature};
use skde::delay_encryption::{decrypt, SkdeParams};

use super::{get_encrypted_transaction_list, get_raw_transaction_info_list};
use crate::{
    client::distributed_key_generation::DistributedKeyGenerationClient,
    error::Error,
    state::AppState,
    types::*,
    util::{fetch_encrypted_transaction, fetch_raw_transaction_info},
};

pub async fn skde_build_block(
    context: AppState,
    cluster: &Cluster,
    rollup_id: String,
    rollup_block_height: u64,
    transaction_count: u64,
    leader_tx_orderer_address: Address,
    signature: Option<Signature>,
) -> Result<Block, Error> {
    let distributed_key_generation_client = context.distributed_key_generation_client().clone();

    let rollup = Rollup::get(&rollup_id).unwrap();

    let skde_params = distributed_key_generation_client
        .get_skde_params()
        .await
        .unwrap()
        .skde_params;

    let merkle_tree = MerkleTree::new();
    let mut decryption_keys: HashMap<u64, String> = HashMap::new();

    let mut encrypted_transaction_list =
        get_encrypted_transaction_list(&rollup_id, rollup_block_height, transaction_count);
    let raw_transaction_info_list =
        get_raw_transaction_info_list(&rollup_id, rollup_block_height, transaction_count);
    let mut final_raw_transaction_list =
        Vec::<RawTransaction>::with_capacity(transaction_count as usize);
    final_raw_transaction_list.resize(transaction_count as usize, RawTransaction::default());

    for (i, raw_transaction_info) in raw_transaction_info_list.iter().enumerate() {
        match raw_transaction_info {
            Some((rawtransaction, is_direct_sent)) => {
                final_raw_transaction_list[i] = rawtransaction.clone();

                if *is_direct_sent && encrypted_transaction_list[i].is_some() {
                    tracing::error!("Raw transaction and encrypted transaction are both present.");
                }
            }
            None => {
                let mut is_direct_sent = false;
                if encrypted_transaction_list[i].is_some() {
                    let skde_encrypted_transaction = encrypted_transaction_list[i]
                        .as_ref()
                        .cloned()
                        .unwrap()
                        .try_into_skde_transaction()
                        .unwrap();

                    let (raw_transaction, _plain_data) = decrypt_skde_transaction(
                        &skde_encrypted_transaction,
                        distributed_key_generation_client.clone(),
                        &mut decryption_keys,
                        &skde_params,
                    )
                    .await
                    .unwrap();

                    final_raw_transaction_list[i] = raw_transaction;
                } else {
                    match fetch_encrypted_transaction(
                        context.rpc_client(),
                        &cluster,
                        &rollup_id,
                        rollup_block_height,
                        i as u64,
                    )
                    .await
                    {
                        Ok(encrypted_transaction) => {
                            encrypted_transaction_list[i] = Some(encrypted_transaction.clone());

                            let _ = EncryptedTransactionModel::put(
                                &rollup_id,
                                rollup_block_height,
                                i as u64,
                                &encrypted_transaction,
                            );

                            let (raw_transaction, _plain_data) = decrypt_skde_transaction(
                                &encrypted_transaction.try_into_skde_transaction().unwrap(),
                                distributed_key_generation_client.clone(),
                                &mut decryption_keys,
                                &skde_params,
                            )
                            .await
                            .unwrap();

                            final_raw_transaction_list[i] = raw_transaction;
                            is_direct_sent = false;
                        }
                        Err(_) => {
                            let (raw_transaction, is_direct_sent_result) =
                                fetch_raw_transaction_info(
                                    context.rpc_client(),
                                    &cluster,
                                    &rollup_id,
                                    rollup_block_height,
                                    i as u64,
                                )
                                .await
                                .unwrap();
                            final_raw_transaction_list[i] = raw_transaction;
                            is_direct_sent = is_direct_sent_result;
                        }
                    }
                }

                let _ = RawTransactionModel::put(
                    &rollup_id,
                    rollup_block_height,
                    i as u64,
                    final_raw_transaction_list[i].clone(),
                    is_direct_sent,
                )
                .unwrap();
            }
        }

        merkle_tree
            .add_data(
                final_raw_transaction_list[i]
                    .raw_transaction_hash()
                    .as_ref(),
            )
            .await;
    }

    merkle_tree.finalize_tree().await;
    let block_commitment = merkle_tree.get_merkle_root().await;

    let signature = if signature.is_some() {
        signature.unwrap()
    } else {
        let signer = context.get_signer(rollup.platform).await.unwrap();
        signer.sign_message(block_commitment).unwrap()
    };

    let block = Block::new(
        rollup_block_height,
        encrypted_transaction_list,
        final_raw_transaction_list,
        signature,
        BlockCommitment::from(block_commitment),
        leader_tx_orderer_address,
    );

    Block::put(&block, &rollup_id, rollup_block_height).unwrap();

    tracing::info!(
        "Block built - block_height: {:?} / transaction_count: {:?}",
        block.block_height,
        block.raw_transaction_list.len()
    );

    Ok(block)
}

async fn decrypt_skde_transaction(
    skde_encrypted_transaction: &SkdeEncryptedTransaction,
    distributed_key_generation_client: DistributedKeyGenerationClient,
    decryption_keys: &mut HashMap<u64, String>,
    skde_params: &SkdeParams,
) -> Result<(RawTransaction, PlainData), Error> {
    let decryption_key_id = skde_encrypted_transaction.key_id;

    // Fetch or insert the decryption key
    let decryption_key = match decryption_keys.entry(decryption_key_id) {
        std::collections::hash_map::Entry::Vacant(entry) => {
            tracing::info!("Fetching decryption key for key_id: {}", decryption_key_id);

            let get_decryption_key_response = distributed_key_generation_client
                .get_decryption_key(decryption_key_id)
                .await
                .map_err(Error::DistributedKeyGeneration)?;

            let inserted_key = entry.insert(get_decryption_key_response.decryption_key.clone());
            inserted_key.clone()
        }
        std::collections::hash_map::Entry::Occupied(entry) => entry.get().clone(),
    };

    match &skde_encrypted_transaction.transaction_data {
        TransactionData::Eth(transaction_data) => {
            let encrypted_data = transaction_data.encrypted_data.clone();

            let decrypted_data = decrypt(skde_params, encrypted_data.as_ref(), &decryption_key)
                .map_err(|e| {
                    tracing::error!(
                        "Decryption failed for key_id: {}: {:?}",
                        decryption_key_id,
                        e
                    );
                    Error::Decryption
                })?;

            let eth_plain_data: EthPlainData =
                serde_json::from_str(&decrypted_data).map_err(|e| {
                    tracing::error!("Failed to parse decrypted data: {:?}", e);
                    Error::Deserialize
                })?;

            let rollup_transaction = transaction_data
                .open_data
                .convert_to_rollup_transaction(&eth_plain_data);

            let eth_raw_transaction = EthRawTransaction::from(to_raw_tx(rollup_transaction));
            let raw_transaction = RawTransaction::from(eth_raw_transaction);

            Ok((raw_transaction, PlainData::from(eth_plain_data)))
        }
        TransactionData::EthBundle(_data) => {
            tracing::warn!("EthBundle transactions are not yet supported.");
            unimplemented!()
        }
    }
}
