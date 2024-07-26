use crate::{
    models::TransactionModel,
    rpc::cluster::{SyncBuildBlock, SyncRequest},
    types::*,
};

pub fn sync_build_block(
    cluster: Cluster,
    rollup_id: RollupId,
    ssal_block_height: BlockHeight,
    rollup_block_height: BlockHeight,
    transaction_order: TransactionOrder,
) {
    tokio::spawn(async move {
        let rpc_method = SyncBuildBlock {
            rollup_id,
            ssal_block_height,
            rollup_block_height,
            transaction_order,
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
    rollup_id: RollupId,
    transaction: TransactionModel,
    order_commitment: OrderCommitment,
) {
    tokio::spawn(async move {
        let rpc_method = SyncRequest {
            rollup_id,
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
