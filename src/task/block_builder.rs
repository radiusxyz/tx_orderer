use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::signature::{Address, Signature};
use skde::{
    delay_encryption::{decrypt, CipherPair, SecretKey},
    SkdeParams,
};
use tracing::info;

use crate::{
    client::liveness::key_management_system::KeyManagementSystemClient, error::Error,
    state::AppState, types::*,
};

pub fn block_builder(
    context: Arc<AppState>,
    rollup_id: String,
    rollup_block_height: u64,
    transaction_counts: u64,
    encrypted_transaction_type: EncryptedTransactionType,

    key_management_system_client: KeyManagementSystemClient,
) {
    info!(
        "rollup_id: {:?}, rollup_block_height: {:?}, transaction_counts: {:?}",
        rollup_id, rollup_block_height, transaction_counts
    );

    tokio::spawn(async move {
        let mut raw_trasnaction_list = Vec::new();
        let mut encrypted_transaction_list = Vec::new();

        match encrypted_transaction_type {
            EncryptedTransactionType::Skde => {
                let mut decryption_keys: HashMap<u64, SecretKey> = HashMap::new();

                for transaction_order in 0..transaction_counts {
                    match RawTransactionModel::get(
                        &rollup_id,
                        rollup_block_height,
                        transaction_order,
                    ) {
                        Ok(raw_transaction) => {
                            raw_trasnaction_list.push(raw_transaction);

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
                                        panic!("error: {:?}", error);
                                    }
                                }
                            };

                            encrypted_transaction_list.push(encrypted_transaction);
                            continue;
                        }
                        Err(error) => {
                            if error.is_none_type() {
                                let encrypted_transaction = EncryptedTransactionModel::get(
                                    &rollup_id,
                                    rollup_block_height,
                                    transaction_order,
                                )
                                .unwrap();

                                encrypted_transaction_list
                                    .push(Some(encrypted_transaction.clone()));

                                match encrypted_transaction {
                                    EncryptedTransaction::Skde(skde_encrypted_transaction) => {
                                        // TODO: update plain_data
                                        let (raw_transaction, plain_data) =
                                            decrypt_skde_transaction(
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

                                        raw_trasnaction_list.push(raw_transaction);
                                    }
                                    EncryptedTransaction::Pvde(_pvde_encrypted_transaction) => {}
                                };
                            }
                        }
                    }
                }
            }
            EncryptedTransactionType::Pvde => {
                unimplemented!()
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
                // )?;
            }
            _ => {}
        }

        // TODO
        let mut block = Block::new(
            rollup_block_height,
            encrypted_transaction_list.clone(),
            RawTransactionList::new(raw_trasnaction_list.clone()),
            Address::from(vec![]),
            Signature::from(vec![]),
            Timestamp::new("0"),
            BlockCommitment::from(vec![]),
        );

        block.raw_transaction_list = RawTransactionList::new(raw_trasnaction_list);
        block.block_height = rollup_block_height;

        // TODO:

        BlockModel::put(&rollup_id, rollup_block_height, &block).unwrap();
    });
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
