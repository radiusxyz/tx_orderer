use std::time::Duration;

use radius_sequencer_sdk::liveness::{
    subscriber::Subscriber,
    types::{Events, Ssal::SsalEvents},
};
use sequencer::types::{Address, ClusterType, ProposerSetId};
use tokio::time::sleep;
use tracing::info;

use crate::{error::Error, models::ClusterModel};

pub fn init(provider_websocket_url: String, liveness_contract_address: String) {
    tokio::spawn(async move {
        loop {
            tracing::info!(
                "Start event listener {} / {}",
                provider_websocket_url,
                liveness_contract_address
            );

            match Subscriber::new(
                provider_websocket_url.clone(),
                liveness_contract_address.clone(),
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

                let _ = initialize_cluster(data.proposerSetId.to_string(), ClusterType::EigenLayer);
            }
            SsalEvents::RegisterSequencer(data) => {
                println!(
                    "RegisterSequencer - Proposer Set ID: {}\nSequencer Address: {}",
                    data.proposerSetId, data.sequencerAddress
                );

                let _ = register_sequencer(
                    data.proposerSetId.to_string(),
                    data.sequencerAddress.to_string().into(),
                );
            }
            SsalEvents::DeregisterSequencer(data) => {
                println!(
                    "DeregisterSequencer - Proposer Set ID: {}\nSequencer Address: {}",
                    data.proposerSetId, data.sequencerAddress
                );

                let _ = deregister_sequencer(
                    data.proposerSetId.to_string(),
                    data.sequencerAddress.to_string().into(),
                );
            }
        },
    }
}

pub fn initialize_cluster(
    proposer_set_id: ProposerSetId,
    cluster_type: ClusterType,
) -> Result<(), Error> {
    info!("initialize_cluster: {:?}", proposer_set_id);

    let cluster_model = ClusterModel::new(proposer_set_id, cluster_type);

    let _ = cluster_model.put()?;

    Ok(())
}

pub fn register_sequencer(
    proposer_set_id: ProposerSetId,
    sequencer_address: Address,
) -> Result<(), Error> {
    info!(
        "register_sequencer: {:?} / {:?}",
        proposer_set_id, sequencer_address
    );

    let mut cluster_model = ClusterModel::get_mut(&proposer_set_id)?;

    cluster_model
        .sequencer_addresses
        .insert(sequencer_address, true);

    let _ = cluster_model.update()?;

    Ok(())
}

pub fn deregister_sequencer(
    proposer_set_id: ProposerSetId,
    sequencer_address: Address,
) -> Result<(), Error> {
    info!(
        "deregister_sequencer: {:?} / {:?}",
        proposer_set_id, sequencer_address
    );

    let mut cluster_model = ClusterModel::get_mut(&proposer_set_id).unwrap();

    cluster_model.sequencer_addresses.remove(&sequencer_address);

    let _ = cluster_model.update()?;

    Ok(())
}
