use std::time::Duration;

use radius_sequencer_sdk::liveness::{
    subscriber::Subscriber,
    types::{Events, Ssal::SsalEvents},
};
use tokio::time::sleep;

pub fn init(liveness_provider_websocket_url: String, contract_address: String) {
    tokio::spawn(async move {
        loop {
            tracing::info!(
                "Start event listener {} / {}",
                liveness_provider_websocket_url,
                contract_address
            );

            match Subscriber::new(
                liveness_provider_websocket_url.clone(),
                contract_address.clone(),
            ) {
                Ok(subscriber) => match subscriber.initialize_event_handler(callback, ()).await {
                    Ok(_) => {
                        tracing::info!("Successfully initialized the event listener.");
                    }
                    Err(err) => {
                        tracing::warn!("Failed to initialize the event listener: {}", err);
                    }
                },
                Err(err) => {
                    tracing::warn!("Failed to initialize the event listener: {}", err);
                }
            }

            sleep(Duration::from_secs(3)).await;
            tracing::warn!("Reconnecting the event listener..");
        }
    });
}

async fn callback(event: Events, _context: ()) {
    match event {
        Events::Block(_) => {}
        Events::SsalEvents(ssal_events) => match ssal_events {
            SsalEvents::InitializeProposerSet(data) => {
                println!(
                    "InitializeProposerSet - Owner: {}\nProposer Set ID: {}",
                    data.owner, data.proposerSetId
                );

                // TODO: Implement the logic.
            }
            SsalEvents::RegisterSequencer(data) => {
                println!(
                    "RegisterSequencer - Proposer Set ID: {}\nSequencer Address: {}",
                    data.proposerSetId, data.sequencerAddress
                );

                // TODO: Implement the logic.
            }
            SsalEvents::DeregisterSequencer(data) => {
                println!(
                    "DeregisterSequencer - Proposer Set ID: {}\nSequencer Address: {}",
                    data.proposerSetId, data.sequencerAddress
                );

                // TODO: Implement the logic.
            }
        },
    }
}
