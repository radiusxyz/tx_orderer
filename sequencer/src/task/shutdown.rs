use ssal::avs::SsalClient;
use tokio::time::{sleep, Duration};

use crate::{
    task::TraceExt,
    types::{SequencerList, BLOCK_MARGIN},
};

pub fn init(ssal_client: SsalClient) {
    tracing::warn!("Shutdown in progress..");

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await;
            let current_block_height = ssal_client.get_block_height().await.ok_or_trace();

            if let Some(block_height) = current_block_height {
                let sequencer_list = SequencerList::get(block_height - BLOCK_MARGIN).ok();

                if let Some(sequencer_list) = sequencer_list {
                    match sequencer_list
                        .into_inner()
                        .into_iter()
                        .find(|(address, _rpc_url)| *address == ssal_client.address())
                    {
                        Some(_) => continue,
                        None => {
                            ssal_client
                                .seeder_client()
                                .deregister(ssal_client.address())
                                .await
                                .ok_or_trace();

                            tracing::info!(
                                "Shutting down the sequencer at block_height: {}",
                                block_height
                            );
                            break;
                        }
                    }
                }
            }
        }

        std::process::exit(0);
    });
}
