use json_rpc::RpcClient;

use crate::{rpc::external::SyncTransaction, types::*};

pub fn init(
    transaction: Transaction,
    order_commitment: OrderCommitment,
    cluster_metadata: ClusterMetadata,
) {
    let rpc_method = SyncTransaction {
        transaction,
        order_commitment,
    };

    // Fire and forget.
    tokio::spawn(async move {
        for (_public_key, rpc_address) in cluster_metadata.into_followers() {
            if let Some(rpc_address) = rpc_address {
                let rpc_method = rpc_method.clone();
                tokio::spawn(async move {
                    let rpc_client = RpcClient::new(rpc_address, 1).unwrap();
                    let _ = rpc_client.request(rpc_method).await;
                });
            }
        }
    });
}
