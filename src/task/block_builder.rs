use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use radius_sdk::json_rpc::client::{Id, RpcClient};
use sha3::{Digest, Keccak256};
use skde::delay_encryption::{decrypt, SecretKey, SkdeParams};
use tracing::info;

use crate::{
    client::{liveness::key_management_system::KeyManagementSystemClient, validation},
    error::Error,
    rpc::external::GetEncryptedTransactionWithOrderCommitment,
    state::AppState,
    types::*,
};

/// Block-builder task implements block-building mechanism for different
/// transaction types in the following order:
///
/// 1. Iterate over transactions for a given rollup ID and the block height.
/// 2. Fetch missing transactions.
/// 3. - PVDE => Decrypt missing raw transactions from other sequencers.
///    - SKDE => Decrypt the transaction with a decryption key.
/// 4. Build the block with the list of raw transactions.
/// 5. (Leader) Submit the block commitment.
pub fn block_builder(
    context: Arc<AppState>,
    rollup_id: String,
    rollup_encrypted_transaction_type: EncryptedTransactionType,
    rollup_block_height: u64,
    transaction_count: u64,
    cluster: Cluster,
) {
    info!(
        "build block - block number: {:?}, transaction count: {:?}",
        rollup_block_height, transaction_count
    );

    match rollup_encrypted_transaction_type {
        EncryptedTransactionType::Pvde => {
            block_builder_pvde(context, rollup_id, rollup_block_height, transaction_count);
        }
        EncryptedTransactionType::Skde => {
            block_builder_skde(
                context,
                rollup_id,
                rollup_block_height,
                transaction_count,
                cluster,
            );
        }
        EncryptedTransactionType::NotSupport => unimplemented!(),
    }
}

// Keccak256 hash function wrapper
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

pub fn is_multiple_of_two(transaction_hash_list: &VecDeque<[u8; 32]>) -> bool {
    match transaction_hash_list.len() % 2 {
        0 => true,
        _ => false,
    }
}

// Function to construct Merkle root from a list of transaction hashes (leaves)
pub fn merkle_root(transaction_hash_list: Vec<RawTransactionHash>) -> BlockCommitment {
    let mut leaves: VecDeque<[u8; 32]> = transaction_hash_list
        .into_iter()
        .map(|transaction_hash| transaction_hash.as_bytes().unwrap())
        .collect();

    if leaves.is_empty() {
        BlockCommitment::default()
    } else {
        while leaves.len() > 1 {
            if is_multiple_of_two(&leaves) {
                leaves = merkle_proof(&mut leaves);
            } else {
                leaves.push_back([0_u8; 32]);
                leaves = merkle_proof(&mut leaves);
            }
        }

        // Return the root
        // # Safety
        // It is safe to call unwrap(). because the loop exits with leaves.len() == 1;
        BlockCommitment::from(leaves.pop_front().unwrap())
    }
}

pub fn merkle_proof(leaves: &mut VecDeque<[u8; 32]>) -> VecDeque<[u8; 32]> {
    let mut new_leaves = VecDeque::<[u8; 32]>::new();

    // # Safety
    // It is safe to call unwrap() twice for each rep because we have set the length
    // of the vector to be even.
    while !leaves.is_empty() {
        let l1 = leaves.pop_front().unwrap();
        let l2 = leaves.pop_front().unwrap();
        let combined = keccak256(&[l1, l2].concat());
        new_leaves.push_back(combined);
    }

    new_leaves
}

// pub fn merkle_proof(leaves: Vec<[u8; 32]>, index: usize) -> Vec<[u8; 32]> {
//     let mut proof = Vec::new();
//     let mut tree_level = leaves;
//     let mut idx = index;

//     while tree_level.len() > 1 {
//         if tree_level.len() % 2 == 1 {
//             tree_level.push(*tree_level.last().unwrap()); // duplicate last
// leaf                                                           // if odd
//         }

//         // Add sibling hash to the proof
//         if idx % 2 == 0 {
//             proof.push(tree_level[idx + 1]);
//         } else {
//             proof.push(tree_level[idx - 1]);
//         }

//         // Move to the next level
//         let mut next_level = Vec::new();
//         for i in (0..tree_level.len()).step_by(2) {
//             let combined = [tree_level[i], tree_level[i + 1]].concat();
//             next_level.push(keccak256(&combined));
//         }
//         tree_level = next_level;
//         idx /= 2;
//     }

//     proof
// }

pub fn block_builder_skde(
    context: Arc<AppState>,
    rollup_id: String,
    rollup_block_height: u64,
    transaction_count: u64,
    cluster: Cluster,
) {
    let key_management_system_client = context.key_management_system_client().clone();

    tokio::spawn(async move {
        let mut raw_transaction_list =
            Vec::<RawTransaction>::with_capacity(transaction_count as usize);
        let mut encrypted_transaction_list =
            Vec::<Option<EncryptedTransaction>>::with_capacity(transaction_count as usize);
        let mut transaction_hash_list =
            Vec::<RawTransactionHash>::with_capacity(transaction_count as usize);

        let mut decryption_keys: HashMap<u64, SecretKey> = HashMap::new();

        if transaction_count > 0 {
            // 1. Iterate over transactions for a given rollup ID and the block height.
            for transaction_order in 0..transaction_count {
                let mut encrypted_transaction = match EncryptedTransactionModel::get(
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

                match RawTransactionModel::get(&rollup_id, rollup_block_height, transaction_order) {
                    // If raw transaction exists, add it to the raw transaction list.
                    Ok(raw_transaction) => {
                        raw_transaction_list.push(raw_transaction);
                    }
                    Err(error) => {
                        if error.is_none_type() {
                            if encrypted_transaction.is_none() {
                                // 2. Fetch the missing transaction from other sequencers.
                                encrypted_transaction = Some(
                                    fetch_missing_transaction(
                                        rollup_id.clone(),
                                        rollup_block_height,
                                        transaction_order,
                                        cluster.clone(),
                                    )
                                    .await
                                    .unwrap(),
                                );
                            }

                            match encrypted_transaction.unwrap() {
                                EncryptedTransaction::Skde(skde_encrypted_transaction) => {
                                    let (raw_transaction, _plain_data) = decrypt_skde_transaction(
                                        &skde_encrypted_transaction,
                                        key_management_system_client.clone(),
                                        &mut decryption_keys,
                                        context.skde_params(),
                                    )
                                    .await
                                    .unwrap();

                                    transaction_hash_list
                                        .push(raw_transaction.raw_transaction_hash().clone());

                                    RawTransactionModel::put(
                                        &rollup_id,
                                        rollup_block_height,
                                        transaction_order,
                                        &raw_transaction,
                                    )
                                    .unwrap();

                                    raw_transaction_list.push(raw_transaction);
                                }
                                _ => {
                                    panic!("error: {:?}", error);
                                }
                            }
                        }
                    }
                }
            }
        }

        let signer = context.get_signer(Platform::Ethereum).await.unwrap();
        let address = signer.address().clone();
        let signature = signer.sign_message("").unwrap(); // TODO: set the message.

        let block_commitment = merkle_root(transaction_hash_list);

        let block = Block::new(
            rollup_block_height,
            encrypted_transaction_list.clone(),
            raw_transaction_list.clone(),
            address,
            signature,
            block_commitment.clone(),
            cluster.is_leader(rollup_block_height),
        );

        Block::put(&block, &rollup_id, rollup_block_height).unwrap();

        if cluster.is_leader(rollup_block_height) {
            let rollup = Rollup::get(&rollup_id).unwrap();

            let validation_info =
                ValidationInfoPayload::get(rollup.platform(), rollup.service_provider()).unwrap();

            match validation_info {
                // TODO: we have to manage the nonce for the register block commitment.
                ValidationInfoPayload::EigenLayer(_) => {
                    let validation_client: validation::eigenlayer::ValidationClient = context
                        .get_validation_client(rollup.platform(), rollup.service_provider())
                        .await
                        .unwrap();

                    let _ = validation_client
                        .publisher()
                        .register_block_commitment(
                            block_commitment,
                            rollup_block_height,
                            rollup.rollup_id(),
                            rollup.cluster_id(),
                        )
                        .await;
                }
                ValidationInfoPayload::Symbiotic(_) => {
                    let validation_client: validation::symbiotic::ValidationClient = context
                        .get_validation_client(rollup.platform(), rollup.service_provider())
                        .await
                        .unwrap();

                    let _ = validation_client
                        .publisher()
                        .register_block_commitment(
                            rollup.cluster_id(),
                            rollup.rollup_id(),
                            rollup_block_height,
                            block_commitment,
                        )
                        .await;
                }
            }
        }
    });
}

pub fn block_builder_pvde(
    _context: Arc<AppState>,
    _rollup_id: String,
    _rollup_block_height: u64,
    _transaction_count: u64,
) {
    // TODO
    // let raw_transaction =
    // decrypt_transaction(
    //     parameter.encrypted_transaction.
    // clone(),
    //     parameter.time_lock_puzzle.
    // clone(),
    //     context.config().is_using_zkp(),
    //     &Some(PvdeParams::default()),
    // )?;
    // RawTransactionModel::put(
    //     &parameter.rollup_id,
    //     rollup_block_height,
    //     transaction_order,
    //     raw_transaction,
    // )?
    unimplemented!("Block builder for PVDE is unimplemented.")
}

async fn decrypt_skde_transaction(
    skde_encrypted_transaction: &SkdeEncryptedTransaction,
    key_management_system_client: KeyManagementSystemClient,
    decryption_keys: &mut HashMap<u64, SecretKey>,
    skde_params: &SkdeParams,
) -> Result<(RawTransaction, PlainData), Error> {
    let decryption_key_id = skde_encrypted_transaction.key_id();

    let decryption_key = if let std::collections::hash_map::Entry::Vacant(e) =
        decryption_keys.entry(decryption_key_id)
    {
        println!("key_id: {:?}", skde_encrypted_transaction.key_id());

        let decryption_key = SecretKey {
            sk: key_management_system_client
                .get_decryption_key(skde_encrypted_transaction.key_id())
                .await
                .unwrap()
                .decryption_key
                .sk,
        };

        e.insert(decryption_key.clone());
        decryption_key
    } else {
        decryption_keys.get(&decryption_key_id).unwrap().clone()
    };

    match skde_encrypted_transaction.transaction_data() {
        TransactionData::Eth(transaction_data) => {
            let encrypted_data = transaction_data.encrypted_data().clone();

            let decrypted_data =
                decrypt(skde_params, encrypted_data.as_ref(), &decryption_key).unwrap();

            let eth_plain_data: EthPlainData = serde_json::from_str(&decrypted_data).unwrap();

            let rollup_transaction = transaction_data
                .open_data()
                .convert_to_rollup_transaction(&eth_plain_data);

            let eth_raw_transaction = EthRawTransaction::from(to_raw_tx(rollup_transaction));
            let raw_transaction = RawTransaction::from(eth_raw_transaction);

            Ok((raw_transaction, PlainData::from(eth_plain_data)))
        }
        TransactionData::EthBundle(_data) => {
            unimplemented!()
        }
    }
}

// TODO: Add fetch function to fetch missing transactions.
async fn fetch_missing_transaction(
    rollup_id: String,
    rollup_block_height: u64,
    transaction_order: u64,
    cluster: Cluster,
) -> Result<EncryptedTransaction, Error> {
    let others = cluster
        .get_others_rpc_url_list()
        .into_iter()
        .filter_map(|rpc_url| rpc_url)
        .collect();

    let parameter = GetEncryptedTransactionWithOrderCommitment {
        rollup_id,
        rollup_block_height,
        transaction_order,
    };

    let rpc_client = RpcClient::new().unwrap();
    let rpc_response = rpc_client
        .fetch(
            others,
            GetEncryptedTransactionWithOrderCommitment::METHOD_NAME,
            &parameter,
            Id::Null,
        )
        .await
        .unwrap();

    Ok(rpc_response)
}
