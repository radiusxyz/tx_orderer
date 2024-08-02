use std::pin::Pin;

use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use radius_sequencer_sdk::json_rpc::RpcClient;
use serde::{de::DeserializeOwned, ser::Serialize};
use ssal::avs::LivenessClient;

use crate::{
    error::Error,
    models::{EncryptedTransactionModel, RawTransactionModel},
    types::*,
};

pub fn build_block(
    cluster: RollupCluster,
    ssal_client: LivenessClient,
    rollup_block_height: BlockHeight,
    transaction_order: TransactionOrder,
    register_block_commitment: bool,
) {
    tokio::spawn(async move {
        let mut raw_transaction_list: Vec<Transaction> =
            Vec::with_capacity(transaction_order.value() as usize);

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

async fn fetch<P, R>(
    rpc_client_list: &Vec<RpcClient>,
    method: &'static str,
    parameter: P,
) -> Result<R, Error>
where
    P: Clone + Serialize + Send,
    R: DeserializeOwned,
{
    let fused_futures: Vec<Pin<Box<Fuse<_>>>> = rpc_client_list
        .iter()
        .map(|client| Box::pin(client.request::<P, R>(method, parameter.clone()).fuse()))
        .collect();

    let (rpc_response, _): (R, Vec<_>) = select_ok(fused_futures)
        .await
        .map_err(|_| Error::FetchResponse)?;

    Ok(rpc_response)
}
