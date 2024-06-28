use ssal::avs::SsalClient;
use tokio::time::{sleep, Duration};

use crate::{task::TraceExt, types::SequencerList};

const MARGIN: u64 = 3;

pub fn init(ssal_client: SsalClient) {
    tracing::warn!("Shutdown in progress..");

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await;
            let current_block_number = ssal_client.get_block_number().await.ok_or_trace();

            if let Some(block_number) = current_block_number {
                let sequencer_list = SequencerList::get(block_number - MARGIN).ok();

                if let Some(sequencer_list) = sequencer_list {
                    match sequencer_list
                        .into_inner()
                        .into_iter()
                        .find(|(address, _rpc_url)| *address == ssal_client.address())
                    {
                        Some(_) => continue,
                        None => {
                            tracing::info!(
                                "Shutting down the sequencer at block_number: {}",
                                block_number
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
