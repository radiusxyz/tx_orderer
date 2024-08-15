use std::{sync::Arc, time::Duration};

use radius_sequencer_sdk::liveness::{
    subscriber::Subscriber,
    types::{Events, Ssal::SsalEvents},
};
use tokio::time::sleep;
use tracing::info;

use crate::{
    client::SequencerClient,
    error::Error,
    models::LivenessClusterModel,
    types::{Address, ClusterId, SequencerIndex, ServiceType, SyncInfo},
};

pub fn init(sync_info: Arc<SyncInfo>) {
    tokio::spawn(async move {
        loop {
            if sync_info.sequencing_info().contract_address.is_none() {
                tracing::warn!("Radius liveness contract address is not set.");
                return;
            }

            tracing::info!(
                "Start event listener {:?} / {:?}",
                sync_info.sequencing_info().provider_websocket_url.clone(),
                sync_info
                    .sequencing_info()
                    .contract_address
                    .clone()
                    .unwrap()
            );

            let liveness_contract_address = sync_info
                .sequencing_info()
                .contract_address
                .clone()
                .unwrap()
                .clone();

            match Subscriber::new(
                sync_info.sequencing_info().provider_websocket_url.clone(),
                liveness_contract_address.to_string(),
            ) {
                Ok(subscriber) => match subscriber
                    .initialize_event_handler(callback, sync_info.clone())
                    .await
                {
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

async fn callback(event: Events, context: Arc<SyncInfo>) {
    match event {
        Events::Block(_) => {}
        Events::SsalEvents(ssal_events) => match ssal_events {
            SsalEvents::InitializeProposerSet(_data) => {}
            SsalEvents::RegisterSequencer(data) => {
                println!(
                    "RegisterSequencer - Proposer Set ID: {}\nSequencer Address: {}",
                    data.proposerSetId, data.sequencerAddress
                );

                // TODO:
                let _ = register_sequencer(
                    context.clone(),
                    data.proposerSetId.to_string(),
                    0,
                    data.sequencerAddress.to_string().into(),
                );
            }
            SsalEvents::DeregisterSequencer(data) => {
                println!(
                    "DeregisterSequencer - Proposer Set ID: {}\nSequencer Address: {}",
                    data.proposerSetId, data.sequencerAddress
                );

                let _ = deregister_sequencer(
                    context,
                    data.proposerSetId.to_string(),
                    data.sequencerAddress.to_string().into(),
                );
            }
        },
    }
}

pub fn register_sequencer(
    context: Arc<SyncInfo>,
    cluster_id: ClusterId,
    sequencer_index: SequencerIndex,
    sequencer_address: Address,
) -> Result<(), Error> {
    info!(
        "register_sequencer: {:?} / {:?} /{:?}",
        cluster_id,
        ServiceType::Radius,
        sequencer_address
    );

    let mut liveness_cluster_model = LivenessClusterModel::get_mut(
        &context.sequencing_info().platform,
        &ServiceType::Radius,
        &cluster_id,
    )?;

    liveness_cluster_model.add_seqeuncer(sequencer_address.clone());
    liveness_cluster_model.update()?;

    tokio::spawn(async move {
        loop {
            match context
                .app_state()
                .seeder_client()
                .get_rpc_url(&sequencer_address)
                .await
            {
                Ok(rpc_url) => match SequencerClient::new(rpc_url) {
                    Ok(rpc_client) => {
                        if let Ok(cluster) = context.app_state().get_cluster(&cluster_id) {
                            cluster
                                .add_sequencer_rpc_client(
                                    sequencer_index,
                                    sequencer_address,
                                    rpc_client,
                                )
                                .await;

                            tracing::info!(
                                "Successfully added SequencerClient for id: {}",
                                cluster_id
                            );
                            break;
                        } else {
                            tracing::warn!("Failed to get cluster for id: {}", cluster_id);
                        }
                    }
                    Err(err) => {
                        tracing::error!("Failed to create SequencerClient: {}", err);
                    }
                },
                Err(err) => {
                    tracing::warn!("Failed to get rpc url: {}", err);
                }
            };

            sleep(Duration::from_secs(10)).await;
        }
    });

    Ok(())
}

pub async fn deregister_sequencer(
    context: Arc<SyncInfo>,

    cluster_id: ClusterId,
    sequencer_address: Address,
) -> Result<(), Error> {
    info!(
        "deregister_sequencer: {:?} / {:?} /{:?}",
        cluster_id,
        ServiceType::Radius,
        sequencer_address
    );

    let mut liveness_cluster_model = LivenessClusterModel::get_mut(
        &context.sequencing_info().platform,
        &ServiceType::Radius,
        &cluster_id,
    )?;

    liveness_cluster_model.remove_sequencer(&sequencer_address);
    liveness_cluster_model.update()?;

    let cluster = context.app_state().get_cluster(&cluster_id)?;
    cluster.remove_sequencer_rpc_client(sequencer_address).await;

    Ok(())
}
