use tracing::info;

use crate::{
    models::{EncryptedTransactionModel, RawTransactionModel, TransactionModel},
    rpc::cluster::{SyncBlock, SyncTransaction},
    types::*,
};

pub fn sync_block(
    cluster: Cluster,
    rollup_id: RollupId,
    cluster_block_height: BlockHeight,
    rollup_block_height: BlockHeight,
    transaction_order: TransactionOrder,
) {
    tokio::spawn(async move {
        let parameter = SyncBlock {
            rollup_id,
            cluster_block_height,
            rollup_block_height,
            transaction_order,
        };

        // Todo: change unwrap
        let sequencer_rpc_clients = cluster.get_other_sequencer_rpc_clients().await;

        info!(
            "sync_block - parameter: {:?} / rpc_client_count: {:?}",
            parameter,
            sequencer_rpc_clients.len()
        );

        for sequencer_rpc_client in sequencer_rpc_clients {
            let sequencer_rpc_client = sequencer_rpc_client.clone();
            let parameter = parameter.clone();

            tokio::spawn(async move {
                let _ = sequencer_rpc_client.sync_block(parameter).await;
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
        let parameter = SyncTransaction {
            rollup_id,
            transaction,
            order_commitment,
        };
        let rpc_clients = cluster.get_other_sequencer_rpc_clients().await;

        info!(
            "sync_transaction - rpc_client_count: {:?}",
            rpc_clients.len()
        );

        for sequencer_rpc_client in rpc_clients {
            let sequencer_rpc_client = sequencer_rpc_client.clone();
            let parameter = parameter.clone();

            tokio::spawn(async move {
                let _ = sequencer_rpc_client.sync_transaction(parameter).await;
            });
        }
    });
}
