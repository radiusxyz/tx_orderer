use crate::{
    rpc::cluster::{SyncBuildBlock, SyncTransaction},
    types::*,
};

pub fn sync_build_block(
    cluster: Cluster,
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

        for rpc_client in cluster.followers() {
            let rpc_client = rpc_client.clone();
            let rpc_method = rpc_method.clone();

            tokio::spawn(async move {
                let _ = rpc_client
                    .request::<SyncBuildBlock, ()>(SyncBuildBlock::METHOD_NAME, rpc_method)
                    .await;
            });
        }
    });
}

pub fn sync_user_transaction(
    cluster: Cluster,
    transaction: UserTransaction,
    order_commitment: OrderCommitment,
) {
    tokio::spawn(async move {
        let rpc_method = SyncTransaction {
            transaction,
            order_commitment,
        };

        for rpc_client in cluster.followers() {
            let rpc_client = rpc_client.clone();
            let rpc_method = rpc_method.clone();

            tokio::spawn(async move {
                let _ = rpc_client
                    .request::<SyncTransaction, ()>(SyncTransaction::METHOD_NAME, rpc_method)
                    .await;
            });
        }
    });
}
