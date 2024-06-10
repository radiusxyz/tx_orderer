use json_rpc::RpcClient;

use crate::{rpc::external::SyncBuildBlock, types::*};

pub fn init(
    ssal_block_number: SsalBlockNumber,
    rollup_block_number: RollupBlockNumber,
    previous_block_height: u64,
) {
    tokio::spawn(async move {
        let me = Me::get().unwrap();
        let sequencer_list = SequencerList::get(ssal_block_number).unwrap();
        let rpc_method = SyncBuildBlock {
            ssal_block_number,
            rollup_block_number,
            previous_block_height,
        };

        for (public_key, rpc_address) in sequencer_list.into_iter() {
            // Always skip forwarding to myself to avoid redundant handling.
            if me == public_key {
                continue;
            }

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
