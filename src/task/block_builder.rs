use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::signature::{Address, Signature};
use skde::delay_encryption::{decrypt, CipherPair};

use crate::{
    client::liveness::key_management_system::KeyManagementSystemClient, state::AppState, types::*,
};

pub fn block_builder(
    context: Arc<AppState>,
    rollup_id: String,
    rollup_block_height: u64,
    transaction_counts: u64,
    encrypted_transaction_type: EncryptedTransactionType,

    key_management_system_client: KeyManagementSystemClient,
    zkp_params: &ZkpParams,
) {
    let skde_zkp_params = zkp_params.skde_params().cloned();
    tokio::spawn(async move {
        let mut decryption_keys = HashMap::new();
        let mut raw_trasnaction_list = Vec::new();
        let mut encrypted_transaction_list = Vec::new();

        for transaction_order in 0..transaction_counts {
            let Ok(encrypted_transaction) =
                EncryptedTransactionModel::get(&rollup_id, rollup_block_height, transaction_order)
            else {
                continue;
            };

            let mut new_encrypted_transaction = encrypted_transaction.clone();

            match RawTransactionModel::get(&rollup_id, rollup_block_height, transaction_order) {
                Ok(raw_transaction) => {
                    raw_trasnaction_list.push(raw_transaction);
                    continue;
                }
                Err(error) => {
                    if error.is_none_type() {
                        match encrypted_transaction {
                            EncryptedTransaction::Pvde(_pvde_encrypted_transaction) => {
                                // TODO
                                // let raw_transaction = decrypt_transaction(
                                //     parameter.encrypted_transaction.clone(),
                                //     parameter.time_lock_puzzle.clone(),
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
                            EncryptedTransaction::Skde(skde_encrypted_transaction) => {
                                let decryption_key_id = skde_encrypted_transaction.key_id();

                                let decryption_key =
                                    if !decryption_keys.contains_key(&decryption_key_id) {
                                        let decryption_key = key_management_system_client
                                            .get_decryption_key(skde_encrypted_transaction.key_id())
                                            .await
                                            .unwrap()
                                            .key;

                                        decryption_keys
                                            .insert(decryption_key_id, decryption_key.clone());
                                        decryption_key
                                    } else {
                                        decryption_keys.get(&decryption_key_id).unwrap().clone()
                                    };

                                match skde_encrypted_transaction.transaction_data() {
                                    TransactionData::Eth(data) => {
                                        let encrypted_data =
                                            data.encrypted_data().clone().into_inner();
                                        let mut encrypted_data_iter = encrypted_data.split("/");
                                        let c1 = encrypted_data_iter.next().unwrap().to_string();
                                        let c2 = encrypted_data_iter.next().unwrap().to_string();
                                        let cipher_pair = CipherPair { c1, c2 };

                                        let skde_params = skde_zkp_params.clone().unwrap();
                                        let plain_text = decrypt(
                                            &skde_params,
                                            &cipher_pair,
                                            &decryption_key.into(),
                                        )
                                        .unwrap();

                                        let eth_plain_data =
                                            string_to_eth_plain_data(&plain_text).unwrap();

                                        let mut new_eth_transaction_data = data.clone();
                                        new_eth_transaction_data.update_plain_data(eth_plain_data);

                                        let typed_eth_rollup_transaction = new_eth_transaction_data
                                            .convert_to_rollup_transaction()
                                            .unwrap();

                                        let raw_eth_transaction = typed_eth_rollup_transaction
                                            .to_raw_transaction()
                                            .unwrap();

                                        RawTransactionModel::put(
                                            &rollup_id,
                                            rollup_block_height,
                                            transaction_order,
                                            raw_eth_transaction.clone(),
                                        )
                                        .unwrap();

                                        raw_trasnaction_list.push(raw_eth_transaction);

                                        new_encrypted_transaction.update_transaction_data(
                                            TransactionData::Eth(new_eth_transaction_data),
                                        );

                                        EncryptedTransactionModel::put(
                                            &rollup_id,
                                            rollup_block_height,
                                            transaction_order,
                                            &new_encrypted_transaction,
                                        )
                                        .unwrap();
                                        encrypted_transaction_list.push(new_encrypted_transaction);
                                    }
                                    TransactionData::EthBundle(_data) => {
                                        encrypted_transaction_list
                                            .push(new_encrypted_transaction.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // TODO
        let mut block = Block::new(
            rollup_block_height,
            EncryptedTransactionList::new(encrypted_transaction_list.clone()),
            RawTransactionList::new(raw_trasnaction_list.clone()),
            Address::from(vec![]),
            Signature::from(vec![]),
            Timestamp::new("0"),
            BlockCommitment::from(vec![]),
        );

        block.encrypted_transaction_list =
            EncryptedTransactionList::new(encrypted_transaction_list);
        block.raw_transaction_list = RawTransactionList::new(raw_trasnaction_list);
        block.block_height = rollup_block_height;

        BlockModel::put(&rollup_id, rollup_block_height, &block).unwrap();
    });
}
