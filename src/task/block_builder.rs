use std::collections::HashMap;

use crate::{client::liveness::key_management_system::KeyManagementSystemClient, types::*};

pub fn block_builder(
    rollup_id: String,
    rollup_block_height: u64,
    transaction_counts: u64,
    encrypted_transaction_type: EncryptedTransactionType,

    key_management_system_client: KeyManagementSystemClient,
) {
    tokio::spawn(async move {
        // TODO: move to builder
        match encrypted_transaction_type {
            EncryptedTransactionType::Pvde => {
                // TODO:
                // unimplemented!();
            }
            EncryptedTransactionType::Skde => {
                let mut decryption_keys = HashMap::new();

                for transaction_order in 0..transaction_counts {
                    let encrypted_transaction = EncryptedTransactionModel::get(
                        &rollup_id,
                        rollup_block_height,
                        transaction_order,
                    )
                    .unwrap();

                    match encrypted_transaction {
                        EncryptedTransaction::Skde(skde_encrypted_transaction) => {
                            match RawTransactionModel::get(
                                &rollup_id,
                                rollup_block_height,
                                transaction_order,
                            ) {
                                Ok(_raw_transaction) => continue,
                                Err(error) => {
                                    if error.is_none_type() {
                                        let decryption_key_id = skde_encrypted_transaction.key_id();

                                        let decryption_key;
                                        if !decryption_keys.contains_key(&decryption_key_id) {
                                            decryption_key = key_management_system_client
                                                .get_decryption_key(
                                                    skde_encrypted_transaction.key_id(),
                                                )
                                                .await
                                                .unwrap()
                                                .key;

                                            decryption_keys
                                                .insert(decryption_key_id, decryption_key.clone());
                                        } else {
                                            decryption_key = decryption_keys
                                                .get(&decryption_key_id)
                                                .unwrap()
                                                .clone();
                                        }

                                        // let raw_transaction =

                                        // RawTransactionModel::put(
                                        //     &rollup_id,
                                        //     rollup_block_height,
                                        //     transaction_order,
                                        //     &raw_transaction,
                                        // )
                                        // .unwrap();

                                        // BlockModel::put(&rollup_id,
                                        // rollup_block_height).unwrap();

                                        // block.
                                    }
                                }
                            }
                        }
                        EncryptedTransaction::Pvde(_transaction) => continue,
                    }
                }
            }
            EncryptedTransactionType::NotSupport => {
                // TODO:
            }
        };
    });
}
