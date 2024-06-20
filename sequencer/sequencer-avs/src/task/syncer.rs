use json_rpc::RpcClient;
use serde::ser::Serialize;
use ssal::avs::types::Address;

use crate::{
    rpc::external::{SyncBuildBlock, SyncUserTransaction},
    types::*,
};

pub fn sync_build_block(
    ssal_block_number: u64,
    rollup_block_number: u64,
    previous_block_height: u64,
    cluster_metadata: ClusterMetadata,
) {
    let rpc_method = SyncBuildBlock {
        ssal_block_number,
        rollup_block_number,
        previous_block_height,
    };

    // TODO: Fetch my address
    let me = Address::ZERO;
    let sequencer_list = cluster_metadata.sequencer_list.into_iter_without(me);

    for (_address, rpc_url) in sequencer_list.into_iter() {
        if let Some(rpc_url) = rpc_url {
            fire_and_forget::<SyncBuildBlock>(
                rpc_url,
                SyncBuildBlock::METHOD_NAME,
                rpc_method.clone(),
            );
        }
    }
}

pub fn sync_user_transaction(
    transaction: UserTransaction,
    order_commitment: OrderCommitment,
    cluster_metadata: ClusterMetadata,
) {
    tokio::spawn(async move {
        let rpc_method = SyncUserTransaction {
            transaction,
            order_commitment,
        };

        // TODO: Fetch my address
        let me = Address::ZERO;
        let sequencer_list = cluster_metadata.sequencer_list.into_iter_without(me);

        for (_address, rpc_url) in sequencer_list.into_iter() {
            if let Some(rpc_url) = rpc_url {
                fire_and_forget::<SyncUserTransaction>(
                    rpc_url,
                    SyncUserTransaction::METHOD_NAME,
                    rpc_method.clone(),
                );
            }
        }
    });
}

// Wrapper function around the fire and forget operation.
fn fire_and_forget<T>(sequencer_rpc_url: String, method_name: &'static str, rpc_method: T)
where
    T: Clone + Serialize + Send + 'static,
{
    tokio::spawn(async move {
        let rpc_client = RpcClient::new(sequencer_rpc_url, 1).unwrap();
        let _ = rpc_client.request::<T, ()>(method_name, rpc_method).await;
    });
}
