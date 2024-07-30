use std::time::Duration;

pub use database::database;
use radius_sequencer_sdk::liveness::{
    subscriber::Subscriber,
    types::{Events, Ssal::SsalEvents},
};
use sequencer::types::Address;
use tokio::time::sleep;
use tracing::info;

use crate::models::{ClusterModel, ProposerSetId};

pub fn init(provider_websocket_url: String, contract_address: String) {
    tokio::spawn(async move {
        loop {
            tracing::info!(
                "Start event listener {} / {}",
                provider_websocket_url,
                contract_address
            );

            match Subscriber::new(provider_websocket_url.clone(), contract_address.clone()) {
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

                initialize_cluster(data.proposerSetId.to_string());
            }
            SsalEvents::RegisterSequencer(data) => {
                println!(
                    "RegisterSequencer - Proposer Set ID: {}\nSequencer Address: {}",
                    data.proposerSetId, data.sequencerAddress
                );

                register_sequencer(
                    data.proposerSetId.to_string(),
                    data.sequencerAddress.to_string().into(),
                )
            }
            SsalEvents::DeregisterSequencer(data) => {
                println!(
                    "DeregisterSequencer - Proposer Set ID: {}\nSequencer Address: {}",
                    data.proposerSetId, data.sequencerAddress
                );

                deregister_sequencer(
                    data.proposerSetId.to_string(),
                    data.sequencerAddress.to_string().into(),
                )
            }
        },
    }
}

fn initialize_cluster(proposer_set_id: ProposerSetId) {
    info!("initialize_cluster: {:?}", proposer_set_id);

    let cluster_model = ClusterModel::new(proposer_set_id);

    let _ = cluster_model.put();
}

fn register_sequencer(proposer_set_id: ProposerSetId, sequencer_address: Address) {
    info!(
        "register_sequencer: {:?} / {:?}",
        proposer_set_id, sequencer_address
    );

    let mut cluster_model = ClusterModel::get_mut(proposer_set_id).unwrap();

    cluster_model
        .sequencer_addresses
        .insert(sequencer_address, true);

    let _ = cluster_model.commit();
}

fn deregister_sequencer(proposer_set_id: ProposerSetId, sequencer_address: Address) {
    info!(
        "deregister_sequencer: {:?} / {:?}",
        proposer_set_id, sequencer_address
    );

    let mut cluster_model = ClusterModel::get_mut(proposer_set_id).unwrap();

    cluster_model.sequencer_addresses.remove(&sequencer_address);

    let _ = cluster_model.commit();
}
