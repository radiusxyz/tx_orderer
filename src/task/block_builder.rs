use std::{collections::HashMap, pin::Pin, sync::Arc};

use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use radius_sequencer_sdk::{
    json_rpc::RpcClient,
    signature::{Address, Signature},
};
use skde::{
    delay_encryption::{decrypt, CipherPair, SecretKey},
    SkdeParams,
};
use tracing::info;

use crate::{
    client::liveness::key_management_system::KeyManagementSystemClient, error::Error,
    rpc::external::GetEncryptedTransactionWithOrderCommitment, state::AppState, types::*,
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
        "rollup_id: {:?}, rollup_block_height: {:?}, transaction_count: {:?}",
        rollup_id, rollup_block_height, transaction_count
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

        let mut decryption_keys: HashMap<u64, SecretKey> = HashMap::new();

        // 1. Iterate over transactions for a given rollup ID and the block height.
        for transaction_order in 0..transaction_count {
            let encrypted_transaction = match EncryptedTransactionModel::get(
                &rollup_id,
                rollup_block_height,
                transaction_order,
            ) {
                Ok(encrypted_transaction) => Some(encrypted_transaction),
                Err(error) => {
                    if error.is_none_type() {
                        // 2. Fetch the missing transaction from other sequencers.
                        match fetch_missing_transaction(
                            rollup_id.clone(),
                            rollup_block_height,
                            transaction_order,
                            cluster.clone(),
                        )
                        .await
                        {
                            Ok(encrypted_transaction) => Some(encrypted_transaction),
                            Err(error) => panic!("block_builder: {:?}", error),
                        }
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
                        match encrypted_transaction {
                            Some(encrypted_transaction) => match encrypted_transaction {
                                EncryptedTransaction::Skde(skde_encrypted_transaction) => {
                                    let (raw_transaction, _plain_data) = decrypt_skde_transaction(
                                        &skde_encrypted_transaction,
                                        key_management_system_client.clone(),
                                        &mut decryption_keys,
                                        context.skde_params(),
                                    )
                                    .await
                                    .unwrap();

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
                            },
                            None => {
                                unimplemented!("Sync encrypted transaction.")
                            }
                        }
                    }
                }
            }
        }

        let signer = context.get_signer(Platform::Ethereum).await.unwrap();
        let address = signer.address().clone();
        let signature = signer.sign_message("").unwrap(); // TODO: set the message.
        let block_commitment = BlockCommitment::from(vec![]);

        let block = Block::new(
            rollup_block_height,
            encrypted_transaction_list.clone(),
            raw_transaction_list.clone(),
            address,
            signature,
            Timestamp::new("0"),
            block_commitment,
        );

        BlockModel::put(&rollup_id, rollup_block_height, &block).unwrap();

        if cluster.is_leader(rollup_block_height) {
            // TODO: (Leader) Register the block commitment.
        }
    });
}

pub fn block_builder_pvde(
    context: Arc<AppState>,
    rollup_id: String,
    rollup_block_height: u64,
    transaction_count: u64,
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

    let decryption_key = if !decryption_keys.contains_key(&decryption_key_id) {
        println!("key_id(): {:?}", skde_encrypted_transaction.key_id());

        let decryption_key = SecretKey {
            sk: key_management_system_client
                .get_decryption_key(skde_encrypted_transaction.key_id())
                .await
                .unwrap()
                .decryption_key
                .sk,
        };

        decryption_keys.insert(decryption_key_id, decryption_key.clone());
        decryption_key
    } else {
        decryption_keys.get(&decryption_key_id).unwrap().clone()
    };

    match skde_encrypted_transaction.transaction_data() {
        TransactionData::Eth(transaction_data) => {
            let encrypted_data = transaction_data.encrypted_data().clone().into_inner();

            println!("encrypted_data: {:?}", encrypted_data);

            let mut encrypted_data_iter = encrypted_data.split("/");

            let c1 = encrypted_data_iter.next().unwrap().to_string();
            let c2 = encrypted_data_iter.next().unwrap().to_string();

            println!("c1: {:?}", c1);
            println!("c2: {:?}", c2);

            let cipher_text = CipherPair { c1, c2 };

            // let  = skde_zkp_params.clone().unwrap();
            let decrypted_data = decrypt(skde_params, &cipher_text, &decryption_key).unwrap();

            let eth_plain_data: EthPlainData = serde_json::from_str(&decrypted_data).unwrap();

            let rollup_transaction = transaction_data
                .open_data()
                .convert_to_rollup_transaction(&eth_plain_data);

            let eth_raw_transaction = EthRawTransaction::from(to_raw_tx(rollup_transaction));
            let raw_transaction = RawTransaction::from(eth_raw_transaction);

            // print!("plain_text: {:?}", plain_text);

            // let eth_plain_data = string_to_eth_plain_data(&plain_text).unwrap();

            // let rollup_transaction =
            // transaction_data.convert_to_rollup_transaction().unwrap();
            // let raw_transaction = rollup_transaction.to_raw_transaction().unwrap();

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
    let rpc_client_list: Vec<RpcClient> = cluster
        .get_others_rpc_url_list()
        .into_iter()
        .filter_map(|rpc_url| match rpc_url {
            Some(rpc_url) => RpcClient::new(rpc_url).ok(),
            None => None,
        })
        .collect();

    let method = GetEncryptedTransactionWithOrderCommitment::METHOD_NAME;
    let parameter = GetEncryptedTransactionWithOrderCommitment {
        rollup_id,
        rollup_block_height,
        transaction_order,
    };

    let fused_futures: Vec<Pin<Box<Fuse<_>>>> = rpc_client_list
        .iter()
        .map(|client| Box::pin(client.request(method, parameter.clone()).fuse()))
        .collect();

    let (rpc_response, _): (EncryptedTransaction, Vec<_>) = select_ok(fused_futures)
        .await
        .map_err(|_| Error::FetchResponse)?;

    Ok(rpc_response)
}
