use ::ssal::ethereum::{types::*, SsalClient, SsalListener};
use tokio::time::{sleep, Duration};

use crate::{config::Config, error::Error, types::*};

pub async fn init(config: &Config, context: &SsalClient) -> Result<(), Error> {
    let ssal_rpc_address = config.ssal_rpc_address.clone();
    let contract_address = config.contract_address.clone();
    let context = context.clone();

    tokio::spawn(async move {
        loop {
            let event_listener = SsalListener::init(&ssal_rpc_address, &contract_address)
                .await
                .unwrap();

            initialize_block_subscriber(event_listener.clone(), context.clone());

            initialize_event_subscriber(event_listener.clone(), context.clone());

            tracing::error!("SsalListener disconnected. Retrying..");
            sleep(Duration::from_secs(3)).await;
        }
    });

    Ok(())
}

fn initialize_block_subscriber(event_listener: SsalListener, context: SsalClient) {
    tokio::spawn(async move {
        event_listener
            .block_subscriber(on_new_block, context.clone())
            .await
            .unwrap_or_else(|error| tracing::error!("{}", error));
    });
}

fn initialize_event_subscriber(event_listener: SsalListener, context: SsalClient) {
    let event_listener = event_listener.clone();

    tokio::spawn(async move {
        event_listener
            .event_subscriber(on_new_event, context.clone())
            .await
            .unwrap_or_else(|error| tracing::error!("{}", error));
    });
}

async fn on_new_block(block: Block<H256>, context: SsalClient) {
    if let Some(block_number) = block.number {
        let block_number = block_number.as_u64();
        tracing::info!("{}", block_number);

        match context.get_sequencer_list(block_number).await {
            Ok(sequencer_list) => {
                SsalBlockNumber::from(block_number)
                    .put()
                    .unwrap_or_else(|error| tracing::error!("{}", error));
                SequencerList::from(sequencer_list)
                    .put(block_number.into())
                    .unwrap_or_else(|error| tracing::error!("{}", error))
            }
            Err(error) => tracing::error!("{}", error),
        }
    }
}

async fn on_new_event(event: SsalEvents, context: SsalClient) {
    //
}
