use std::pin::Pin;

use block_commitment::calculate_block_commitment;
use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use json_rpc::RpcClient;
use serde::{de::DeserializeOwned, ser::Serialize};
use ssal::avs::SsalClient;

use super::TraceExt;
use crate::{
    error::Error,
    rpc::cluster::GetTransaction,
    types::{
        BlockCommitment, Cluster, RollupBlock, UserEncryptedTransaction, UserRawTransaction,
        UserTransaction,
    },
};

pub fn build_block(
    ssal_client: SsalClient,
    cluster: Cluster,
    full_node_id: u32,
    rollup_block_number: u64,
    block_length: u64,
    register_block_commitment: bool,
) {
    tokio::spawn(async move {
        let mut block: Vec<UserTransaction> = Vec::with_capacity(block_length as usize);
        let followers = cluster.followers();

        // TODO(jaemin): include the raw tx added by decrypting encrypted tx
        if block_length != 0 {
            for transaction_order in 1..=block_length {
                match (
                    UserEncryptedTransaction::get(
                        full_node_id,
                        rollup_block_number,
                        transaction_order,
                    ),
                    UserRawTransaction::get(full_node_id, rollup_block_number, transaction_order),
                ) {
                    (Err(_), Ok(transaction)) => block.push(UserTransaction::Raw(transaction)),
                    (Ok(transaction), Err(_)) => {
                        block.push(UserTransaction::Encrypted(transaction))
                    }
                    (Ok(encrypted_transaction), Ok(_)) => {
                        block.push(UserTransaction::Encrypted(encrypted_transaction))
                    }
                    (Err(error), Err(_)) => {
                        if error.kind() == database::ErrorKind::KeyDoesNotExist {
                            // Fetch the missing transaction from other sequencers.
                            let rpc_method = GetTransaction {
                                full_node_id,
                                rollup_block_number,
                                transaction_order,
                            };

                            // Stops building the block if the transaction is missing cluster-wide.
                            match fetch::<GetTransaction, UserTransaction>(
                                followers,
                                GetTransaction::METHOD_NAME,
                                rpc_method,
                            )
                            .await
                            {
                                Ok(transaction) => block.push(transaction),
                                _ => break,
                            }
                        } else {
                            // Very unlikely, but we want to see the log.
                            tracing::error!("{}", error);
                        }
                    }
                }
            }
        }

        // Calculate the block_commitment.
        // TODO: Get the seed from SSAL.
        let seed = [0u8; 32];
        let block_commitment: BlockCommitment = calculate_block_commitment(&block, seed).into();
        block_commitment
            .put(full_node_id, rollup_block_number)
            .ok_or_trace();

        // Register the block commitment.
        if register_block_commitment {
            match ssal_client
                .register_block_commitment(block_commitment, rollup_block_number, full_node_id, cluster.id())
                .await
            {
                Ok(_) => tracing::info!("Successfully registered the block commitment.\nRollup block number: {}\nBlock height: {}", rollup_block_number, block_length),
                Err(error) => tracing::error!("{}", error),
            }
        }

        RollupBlock::from(block)
            .put(full_node_id, rollup_block_number)
            .unwrap();
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
