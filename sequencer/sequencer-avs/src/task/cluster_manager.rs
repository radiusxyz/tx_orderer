use std::time::Duration;

use ssal::ethereum::SsalClient;
use tokio::time::sleep;

use crate::types::*;

pub fn init(ssal_client: SsalClient) {
    tokio::spawn(async move {
        let mut last_block_number = ssal_client.get_latest_block_number().await.unwrap();

        loop {
            let block_number = ssal_client.get_latest_block_number().await.unwrap();
            if block_number != last_block_number {
                let sequencer_list: SequencerList = ssal_client
                    .get_sequencer_list(block_number)
                    .await
                    .unwrap()
                    .into();
                tracing::info!("{:?}", sequencer_list);
                sequencer_list.put(block_number.into()).unwrap();
                last_block_number = block_number;
            }
            sleep(Duration::from_secs(3)).await;
        }
    });
}
