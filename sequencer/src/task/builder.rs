use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use radius_sequencer_sdk::json_rpc::RpcClient;
use serde::{de::DeserializeOwned, ser::Serialize};
use ssal::avs::LivenessClient;

use crate::{error::Error, models::BlockModel, types::*};

// TODO: update block commitment to contract
pub fn finalize_block(
    rollup_id: RollupId,
    cluster: Cluster,
    rollup_block_height: BlockHeight,
    transaction_order: TransactionOrder,
) {
    tokio::spawn(async move {
        let mut encrypted_transaction_list: Vec<Option<EncryptedTransaction>> =
            Vec::with_capacity(transaction_order.value() as usize);

        let mut raw_transaction_list: Vec<RawTransaction> =
            Vec::with_capacity(transaction_order.value() as usize);

        // TODO: 1. make encrypted / raw transaction list

        // TODO: 2. make block commitment

        // TODO: 3. set proposer address

        // TODO: 4. set timestamp

        // TODO: 5. make block

        // TODO: 4. sign block (set signature)

        let block = Block::new(
            rollup_block_height,
            EncryptedTransactionList::new(encrypted_transaction_list),
            RawTransactionList::new(raw_transaction_list),
            Address::from("proposer_address"),
            Signature::default(),
            Timestamp::new("1".to_string()),
            vec![0u8; 32].into(),
        );

        let block_model = BlockModel::new(rollup_id, block);

        let _ = block_model.put();

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
