use std::pin::Pin;

use block_commitment::calculate_block_commitment;
use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use json_rpc::RpcClient;
use serde::{de::DeserializeOwned, ser::Serialize};
use ssal::avs::SsalClient;

use crate::{
    error::Error, models::EncryptedTransactionModel, rpc::cluster::GetTransaction, types::*,
};

pub fn build_block(
    ssal_client: SsalClient,
    cluster: Cluster,
    rollup_id: RollupId,
    rollup_block_height: BlockHeight,
    transaction_order: TransactionOrder,
    register_block_commitment: bool,
) {
    tokio::spawn(async move {
        let mut block: Vec<Transaction> = Vec::with_capacity(transaction_order as usize);
        let followers = cluster.followers();

        // TODO(jaemin): include the raw tx added by decrypting encrypted tx
        if transaction_order != 0 {
            for transaction_order in 1..=transaction_order {
                match (
                    EncryptedTransactionModel::get(
                        &rollup_id,
                        &rollup_block_height,
                        &transaction_order,
                    ),
                    RawTransactionModel::get(&rollup_id, &rollup_block_height, &transaction_order),
                ) {
                    (Err(_), Ok(transaction)) => block.push(Transaction::Raw(transaction)),
                    (Ok(transaction), Err(_)) => block.push(Transaction::Encrypted(transaction)),
                    (Ok(encrypted_transaction), Ok(_)) => {
                        block.push(Transaction::Encrypted(encrypted_transaction))
                    }
                    (Err(error), Err(_)) => {
                        if error.kind() == database::ErrorKind::KeyDoesNotExist {
                            // Fetch the missing transaction from other sequencers.
                            let rpc_method = GetTransaction {
                                rollup_id,
                                rollup_block_height,
                                transaction_order,
                            };

                            // Stops building the block if the transaction is missing cluster-wide.
                            match fetch::<GetTransaction, Transaction>(
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
            .put(rollup_id, rollup_block_height)
            .ok_or_trace();

        // Register the block commitment.
        if register_block_commitment {
            match ssal_client
                .register_block_commitment(block_commitment, rollup_block_height, rollup_id, cluster.id())
                .await
            {
                Ok(_) => tracing::info!("Successfully registered the block commitment.\nRollup block number: {}\nBlock height: {}", rollup_block_height, transaction_order),
                Err(error) => tracing::error!("{}", error),
            }
        }

        RollupBlock::from(block)
            .put(rollup_id, rollup_block_height)
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
