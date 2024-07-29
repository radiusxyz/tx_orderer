use std::str::FromStr;

use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::{Block, Log},
    sol,
};
pub use database::database;
use futures::{stream::select_all, Future, Stream, StreamExt, TryStreamExt};
use tokio::time::{sleep, Duration};

use crate::error::Error;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Ssal,
    "src/avs/contract/Ssal.json"
);

pub enum EventType {
    RegisterProposer((Ssal::RegisterSequencer, Log)),
    DeregisterProposer((Ssal::DeregisterSequencer, Log)),
}

pub fn init(provider_rpc_endpoint: &str, contract_address: &str) {
    tokio::spawn(async move {
        loop {
            let websocket = WsConnect::new(provider_rpc_endpoint.as_ref());
            let provider: RootProvider<PubSubFrontend> = ProviderBuilder::new()
                .on_ws(websocket)
                .await
                .map_err(|error| (Error::ConnectEventListener, error))
                .unwrap();

            let contract_address = Address::from_str(contract_address.as_ref())
                .map_err(|error| (Error::ParseContractAddress, error))
                .unwrap();

            let ssal_contract = Ssal::SsalInstance::new(contract_address, provider.clone());

            // registerProposerEvent
            let event = ssal_contract
                .RegisterSequencerEvent_filter()
                .DeregisterSequencerEvent_filter()
                .subscribe()
                .await
                .map_err(|error| (Error::ParseContractAddress, error))?
                .into_stream()
                .boxed()
                .into();

            while let Some(event) = event.next().await {
                event_callback(event).await;
            }

            sleep(Duration::from_secs(3)).await;
            tracing::warn!("Reconnecting the event listener..");
        }
    });
}

async fn event_callback(event_type: EventType) {
    match event_type {
        EventType::RegisterProposer((event, _)) => registered_sequencer(event).await,
        EventType::DeregisterProposer((event, _)) => {
            deregistered_sequencer(event).await;
        }
    }
}

async fn registered_sequencer(registered_event: Ssal::RegisterSequencer) {
    let proposer_set_id = registered_event.proposerSetId.to_string();
    let sequencer_address = registered_event.sequencerAddress.to_string();
    let db = database().unwrap();

    let mut sequencer_list: Vec<Address> = match db.get(&proposer_set_id) {
        Ok(sequencer_list) => sequencer_list,
        Err(_) => vec![],
    };

    sequencer_list.push(sequencer_address.clone());

    db.put(&proposer_set_id, &sequencer_list);
}

async fn deregistered_sequencer(deregistered_event: Ssal::DeregisterSequencer) {
    let proposer_set_id = deregistered_event.proposerSetId.to_string();
    let sequencer_address = deregistered_event.sequencerAddress.to_string();

    let db = database().unwrap();
    let mut sequencer_list: Vec<Address> = match db.get(&proposer_set_id) {
        Ok(sequencer_list) => sequencer_list,
        Err(_) => vec![],
    };

    let index = sequencer_list.iter().position(|x| *x == sequencer_address);

    if let Some(index) = index {
        sequencer_list.remove(index);
    }

    db.put(&proposer_set_id, &sequencer_list);
}
