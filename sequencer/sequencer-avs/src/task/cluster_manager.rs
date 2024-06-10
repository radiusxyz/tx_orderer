use ssal::ethereum::SsalClient;

use crate::types::*;

pub fn init(ssal_client: &SsalClient) {
    let ssal_client = ssal_client.clone();
    tokio::spawn(async move {
        ssal_client
            .block_number_subscriber(handler)
            .await
            .unwrap_or_else(|error| tracing::error!("{}", error));
    });
}

async fn handler(ssal_block_number: u64, ssal_client: SsalClient) {
    tracing::info!("{}", ssal_block_number);
    match ssal_client.get_sequencer_list(ssal_block_number).await {
        Ok(sequencer_list) => {
            SsalBlockNumber::from(ssal_block_number)
                .put()
                .unwrap_or_else(|error| tracing::error!("{}", error));
            SequencerList::from(sequencer_list)
                .put(ssal_block_number.into())
                .unwrap_or_else(|error| tracing::error!("{}", error))
        }
        Err(error) => tracing::error!("{}", error),
    }
}
