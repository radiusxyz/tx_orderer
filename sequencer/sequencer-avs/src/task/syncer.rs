use json_rpc::RpcClient;

use crate::{
    rpc::external::{SyncBuildBlock, SyncUserTransaction},
    types::*,
};

pub fn sync_build_block(
    rpc_client_list: Vec<Option<RpcClient>>,
    ssal_block_number: u64,
    rollup_block_number: u64,
    previous_block_height: u64,
) {
    tokio::spawn(async move {
        let rpc_method = SyncBuildBlock {
            ssal_block_number,
            rollup_block_number,
            previous_block_height,
        };

        for rpc_client in rpc_client_list {
            if let Some(rpc_client) = rpc_client {
                let rpc_method = rpc_method.clone();

                tokio::spawn(async move {
                    let _ = rpc_client
                        .request::<SyncBuildBlock, ()>(SyncBuildBlock::METHOD_NAME, rpc_method)
                        .await;
                });
            }
        }
    });
}

pub fn sync_user_transaction(
    rpc_client_list: Vec<Option<RpcClient>>,
    transaction: UserTransaction,
    order_commitment: OrderCommitment,
) {
    tokio::spawn(async move {
        let rpc_method = SyncUserTransaction {
            transaction,
            order_commitment,
        };

        for rpc_client in rpc_client_list {
            if let Some(rpc_client) = rpc_client {
                let rpc_method = rpc_method.clone();

                tokio::spawn(async move {
                    let _ = rpc_client
                        .request::<SyncUserTransaction, ()>(
                            SyncUserTransaction::METHOD_NAME,
                            rpc_method,
                        )
                        .await;
                });
            }
        }
    });
}
