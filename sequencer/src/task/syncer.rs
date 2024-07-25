use crate::{
    rpc::cluster::{SyncBuildBlock, SyncRequest},
    types::*,
};

pub fn sync_build_block(
    cluster: Cluster,
    full_node_id: u32,
    ssal_block_number: u64,
    rollup_block_number: u64,
    previous_block_length: u64,
) {
    tokio::spawn(async move {
        let rpc_method = SyncBuildBlock {
            full_node_id,
            ssal_block_number,
            rollup_block_number,
            previous_block_length,
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
    full_node_id: u32,
    transaction: UserTransaction,
    order_commitment: OrderCommitment,
) {
    tokio::spawn(async move {
        let rpc_method = SyncRequest {
            full_node_id,
            transaction,
            order_commitment,
        };

        for rpc_client in cluster.followers() {
            let rpc_client = rpc_client.clone();
            let rpc_method = rpc_method.clone();

            tokio::spawn(async move {
                let _ = rpc_client
                    .request::<SyncRequest, ()>(SyncRequest::METHOD_NAME, rpc_method)
                    .await;
            });
        }
    });
}
