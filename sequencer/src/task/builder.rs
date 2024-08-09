use core::time;

use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use radius_sequencer_sdk::{block_commitment::calculate_block_commitment, json_rpc::RpcClient};
use serde::{de::DeserializeOwned, ser::Serialize};
use ssal::avs::LivenessClient;

use crate::{
    error::Error,
    models::{BlockModel, EncryptedTransactionModel, RawTransactionModel},
    types::*,
};

// TODO: update block commitment to contract
pub fn finalize_block(
    rollup_id: RollupId,
    cluster: Cluster,
    rollup_block_height: BlockHeight,
    transaction_order: TransactionOrder,
) {
    tokio::spawn(async move {
        // TODO: 1. make encrypted / raw transaction list
        let mut encrypted_transaction_list: Vec<Option<EncryptedTransaction>> =
            Vec::with_capacity(transaction_order.value() as usize);

        let mut raw_transaction_list: Vec<RawTransaction> =
            Vec::with_capacity(transaction_order.value() as usize);

        // TODO: change
        for i in 0..=transaction_order.value() + 1 {
            EncryptedTransactionModel::get(
                &rollup_id,
                &rollup_block_height,
                &TransactionOrder::new(i),
            )
            .map(|encrypted_transaction| {
                encrypted_transaction_list
                    .push(Some(encrypted_transaction.encrypted_transaction().clone()));
            })
            .unwrap_or_else(|_| encrypted_transaction_list.push(None));

            match RawTransactionModel::get(
                &rollup_id,
                &rollup_block_height,
                &TransactionOrder::new(i),
            ) {
                Ok(raw_transaction) => {
                    raw_transaction_list.push(raw_transaction.raw_transaction().clone());
                }
                // TODO: change
                Err(_) => {}
            }
        }
        // TODO: 2. make block commitment
        // get block_commitment option from config or cluster
        // change calculate logic
        // let seed = [0u8; 32];
        // let block_commitment: BlockCommitment = calculate_block_commitment(block, seed);
        // TODO: Check
        // block_commitment
        //     .put(rollup_id, rollup_block_height)
        //     .ok_or_trace();

        // TODO: 3. set proposer address
        let proposer_address = cluster.node_address();

        // TODO: 4. set timestamp
        let timestamp = Timestamp::new(chrono::Utc::now().timestamp().to_string());

        // TODO: 5. make block
        let block = Block::new(
            rollup_block_height,
            EncryptedTransactionList::new(encrypted_transaction_list),
            RawTransactionList::new(raw_transaction_list),
            proposer_address.clone(),
            Signature::default(),
            timestamp,
            vec![0u8; 32].into(),
        );

        // TODO: 6. sign block (set signature)

        let block_model = BlockModel::new(rollup_id.clone(), block);

        block_model.put().unwrap();

        let encrypted_transaction =
            EncryptedTransactionModel::get(&rollup_id, &rollup_block_height, &transaction_order)
                .unwrap();

        println!(
            "jaemin - encrypted_transaction: {:?}",
            encrypted_transaction
        );

        // let followers = cluster.followers();

        // TODO: Check
        // Register the block commitment.
        // if register_block_commitment {
        //     // Calculate the block_commitment.
        //     // TODO: Get the seed from SSAL.
        //     let seed = [0u8; 32];
        //     let block_commitment: BlockCommitment = calculate_block_commitment(&block, seed).into();

        //     // TODO: Check
        //     // block_commitment
        //     //     .put(rollup_id, rollup_block_height)
        //     //     .ok_or_trace();

        //     match ssal_client
        //         .register_block_commitment(block_commitment, rollup_block_height.into(), rollup_id, cluster.id())
        //         .await
        //     {
        //         Ok(_) => tracing::info!("Successfully registered the block commitment.\nRollup block number: {}\nBlock height: {}", rollup_block_height, transaction_order),
        //         Err(error) => tracing::error!("{}", error),
        //     }
        // }

        // TODO: Check
        // RollupBlock::from(block)
        //     .put(rollup_id, rollup_block_height)
        //     .unwrap();
    });
}
