use json_rpc::RpcClient;

use crate::{rpc::external::SyncTransaction, types::*};

pub fn init() {
    tokio::spawn(async move {});
}

pub fn transaction_syncer(cluster: ClusterMetadata, transaction: Transaction) {
    // tokio::spawn(async move {
    //     for (public_key, address) in cluster.iter() {
    //         if let Some(address) = address {
    //             let rpc_client = RpcClient::new(address, 2).unwrap();
    //             // rpc_client.request(SyncTransaction).await.unwrap();
    //             tokio::spawn(async move {});
    //         }
    //     }
    // });
}
