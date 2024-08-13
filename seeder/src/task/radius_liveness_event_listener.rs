use std::{sync::Arc, time::Duration};

use radius_sequencer_sdk::liveness::{
    subscriber::Subscriber,
    types::{Events, Ssal::SsalEvents},
};
use sequencer::{
    models::{ClusterIdListModel, LivenessClusterModel},
    types::{Address, ClusterId, PlatForm, SequencingFunctionType, SequencingInfo, ServiceType},
};
use tokio::time::sleep;
use tracing::info;

use crate::error::Error;

pub fn init(sequencing_info: SequencingInfo) {
    tokio::spawn(async move {
        loop {
            if sequencing_info.contract_address.is_none() {
                tracing::warn!("Radius liveness contract address is not set.");
                return;
            }

            tracing::info!(
                "Start event listener {:?} / {:?}",
                sequencing_info.provider_websocket_url.clone(),
                sequencing_info.contract_address.clone().unwrap()
            );

            let sequencing_info_context = Arc::new(sequencing_info.clone());

            let liveness_contract_address =
                sequencing_info.contract_address.clone().unwrap().clone();

            match Subscriber::new(
                sequencing_info.provider_websocket_url.clone(),
                liveness_contract_address.to_string(),
            ) {
                Ok(subscriber) => match subscriber
                    .initialize_event_handler(callback, sequencing_info_context)
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

async fn callback(event: Events, context: Arc<SequencingInfo>) {
    match event {
        Events::Block(_) => {}
        Events::SsalEvents(ssal_events) => match ssal_events {
            SsalEvents::InitializeProposerSet(data) => {
                println!(
                    "InitializeProposerSet - Owner: {}\nProposer Set ID: {}",
                    data.owner, data.proposerSetId
                );

                let _ =
                    initialize_cluster(context.platform.clone(), data.proposerSetId.to_string());
            }
            SsalEvents::RegisterSequencer(data) => {
                println!(
                    "RegisterSequencer - Proposer Set ID: {}\nSequencer Address: {}",
                    data.proposerSetId, data.sequencerAddress
                );

                let _ = register_sequencer(
                    context.platform.clone(),
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
                    context.platform.clone(),
                    data.proposerSetId.to_string(),
                    data.sequencerAddress.to_string().into(),
                );
            }
        },
    }
}

pub fn initialize_cluster(platform: PlatForm, cluster_id: ClusterId) -> Result<(), Error> {
    info!("initialize_cluster: {:?}", cluster_id);

    let mut cluster_id_list_model = ClusterIdListModel::entry(
        &platform,
        &SequencingFunctionType::Liveness,
        &ServiceType::Radius,
    )?;

    cluster_id_list_model.add_cluster_id(cluster_id.clone());
    cluster_id_list_model.update()?;

    let cluster_model = LivenessClusterModel::new(platform, ServiceType::Radius, cluster_id);
    cluster_model.put()?;

    Ok(())
}

pub fn register_sequencer(
    platform: PlatForm,

    cluster_id: ClusterId,
    sequencer_address: Address,
) -> Result<(), Error> {
    info!(
        "register_sequencer: {:?} / {:?} /{:?}",
        cluster_id,
        ServiceType::Radius,
        sequencer_address
    );

    let mut liveness_cluster_model =
        LivenessClusterModel::get_mut(&platform, &ServiceType::Radius, &cluster_id)?;

    liveness_cluster_model.add_seqeuncer(sequencer_address);
    liveness_cluster_model.update()?;

    Ok(())
}

pub fn deregister_sequencer(
    platform: PlatForm,

    cluster_id: ClusterId,
    sequencer_address: Address,
) -> Result<(), Error> {
    info!(
        "deregister_sequencer: {:?} / {:?} /{:?}",
        cluster_id,
        ServiceType::Radius,
        sequencer_address
    );

    let mut liveness_cluster_model =
        LivenessClusterModel::get_mut(&platform, &ServiceType::Radius, &cluster_id)?;

    liveness_cluster_model.remove_sequencer(&sequencer_address);
    liveness_cluster_model.update()?;

    Ok(())
}
