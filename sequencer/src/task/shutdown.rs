use ssal::avs::SsalClient;
use tokio::time::{sleep, Duration};

use crate::{models::SequencerList, task::TraceExt, types::BLOCK_MARGIN};

pub fn init(ssal_client: SsalClient) {
    tracing::warn!("Shutdown in progress..");

    tokio::spawn(async move {
        let deregistered_block_height;
        loop {
            let mut current_block_height = ssal_client.get_block_height().await.ok_or_trace();

            if let Some(block_height) = current_block_height {
                ssal_client
                    .seeder_client()
                    .deregister(ssal_client.address())
                    .await
                    .ok_or_trace();

                tracing::info!(
                    "Shutting down the sequencer at block_height: {}",
                    block_height
                );

                deregistered_block_height = block_height;

                break;
            }

            sleep(Duration::from_secs(10)).await;
        }

        loop {
            sleep(Duration::from_secs(10)).await;

            let mut current_block_height = ssal_client.get_block_height().await.ok_or_trace();

            if let Some(current_block_height) = current_block_height {
                let sequencer_list = SequencerList::get(current_block_height - BLOCK_MARGIN).ok();

                if let Some(sequencer_list) = sequencer_list {
                    match sequencer_list
                        .into_inner()
                        .into_iter()
                        .find(|(address, _rpc_url)| *address == ssal_client.address())
                    {
                        Some(_) => break,
                        None => {
                            continue;
                        }
                    }
                }
            }
        }

        std::process::exit(0);
    });
}
