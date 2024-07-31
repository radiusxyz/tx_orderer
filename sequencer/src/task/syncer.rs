use crate::{
    models::TransactionModel,
    rpc::cluster::{SyncBlock, SyncTransaction},
    types::*,
};

pub fn sync_block(
    cluster: Cluster,
    rollup_id: RollupId,
    liveness_block_height: BlockHeight,
    rollup_block_height: BlockHeight,
    transaction_order: TransactionOrder,
) {
    tokio::spawn(async move {
        let rpc_method = SyncBlock {
            rollup_id,
            liveness_block_height,
            rollup_block_height,
            transaction_order,
        };

        for rpc_client in cluster.followers() {
            let rpc_client = rpc_client.clone();
            let rpc_method = rpc_method.clone();

            tokio::spawn(async move {
                let _ = rpc_client
                    .request::<SyncBlock, ()>(SyncBlock::METHOD_NAME, rpc_method)
                    .await;
            });
        }
    });
}

pub fn sync_transaction(
    cluster: Cluster,
    rollup_id: RollupId,
    transaction: TransactionModel,
    order_commitment: OrderCommitment,
) {
    tokio::spawn(async move {
        let rpc_method = SyncTransaction {
            rollup_id,
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
