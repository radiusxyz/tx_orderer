use std::pin::Pin;

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
    rpc::internal::GetTransaction,
    types::{BlockCommitment, Cluster, RollupBlock, UserTransaction},
};

pub fn build_block(
    ssal_client: SsalClient,
    cluster: Cluster,
    rollup_block_number: u64,
    block_height: u64,
    submit_block_commitment: bool,
) {
    tokio::spawn(async move {
        let mut block: Vec<UserTransaction> = Vec::with_capacity(block_height as usize);
        let followers = cluster.followers();

        for transaction_order in 0..block_height {
            match UserTransaction::get(rollup_block_number, transaction_order) {
                Ok(transaction) => block.push(transaction),
                Err(error) => {
                    if error.kind() == database::ErrorKind::KeyDoesNotExist {
                        // Fetch the missing transaction from other sequencers.
                        let rpc_method = GetTransaction {
                            rollup_block_number,
                            transaction_order,
                        };

                        // Stops building the block if the transaction is missing cluster-wide.
                        let transaction = fetch::<GetTransaction, UserTransaction>(
                            followers,
                            GetTransaction::METHOD_NAME,
                            rpc_method,
                        )
                        .await
                        .unwrap();

                        block.push(transaction);
                    } else {
                        // Very unlikely, but we want to see the log.
                        tracing::error!("{}", error);
                    }
                }
            }
        }

        // Calculate the block_commitment.
        // TODO: Get the seed from SSAL.
        let seed = [0u8; 32];
        let block_commitment: BlockCommitment =
            block_commitment::calculate_block_commitment(&block, seed).into();
        block_commitment.put(rollup_block_number).ok_or_trace();

        // Register the block commitment.
        if submit_block_commitment {
            ssal_client
                .register_block_commitment(block_commitment, rollup_block_number, 0, cluster.id())
                .await
                .ok_or_trace();
        }

        RollupBlock::from(block).put(rollup_block_number).unwrap();
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
