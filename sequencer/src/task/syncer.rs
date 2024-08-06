use crate::{
    models::TransactionModel,
    rpc::cluster::{SyncBlock, SyncTransaction},
    types::*,
};

pub fn sync_block(
    cluster: Cluster,
    rollup_id: ClusterId,
    liveness_block_height: BlockHeight,
    rollup_block_height: BlockHeight,
    transaction_order: TransactionOrder,
) {
    tokio::spawn(async move {
        let parameter = SyncBlock {
            rollup_id,
            liveness_block_height,
            rollup_block_height,
            transaction_order,
        };

        for rpc_client in cluster.get_other_sequencer_rpc_clients().await {
            let rpc_client = rpc_client.clone();
            let parameter = parameter.clone();

            tokio::spawn(async move {
                let _ = rpc_client.sync_block(parameter).await;
            });
        }
    });
}

pub fn sync_transaction(
    cluster: Cluster,
    rollup_id: ClusterId,
    transaction: TransactionModel,
    order_commitment: OrderCommitment,
) {
    tokio::spawn(async move {
        let parameter = SyncTransaction {
            rollup_id,
            transaction,
            order_commitment,
        };

        for rpc_client in cluster.get_other_sequencer_rpc_clients().await {
            let rpc_client = rpc_client.clone();
            let parameter = parameter.clone();

            tokio::spawn(async move {
                let _ = rpc_client.sync_transaction(parameter).await;
            });
        }
    });
}
