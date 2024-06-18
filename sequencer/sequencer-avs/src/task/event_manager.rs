use ::ssal::ethereum::{types::*, SsalClient, SsalListener};
use tokio::time::{sleep, Duration};

use crate::{config::Config, error::Error, types::*};

pub fn init(config: &Config, context: ()) {
    let ssal_rpc_address = config.ssal_rpc_address.clone();
    let contract_address = config.contract_address.clone();
    let context = context.clone();

    tokio::spawn(async move {
        loop {
            let event_listener = SsalListener::init(&ssal_rpc_address, &contract_address)
                .await
                .unwrap();

            event_listener
                .with_callback(event_handler, context.clone())
                .await
                .unwrap_or_else(|error| {
                    tracing::error!("{}", error);
                });

            tracing::error!("SsalListener disconnected. Retrying..");
            sleep(Duration::from_secs(3)).await;
        }
    });
}

async fn event_handler(event: SsalEventType, context: ()) {
    match event {
        SsalEventType::NewBlock(block) => on_new_block(block, ()).await,
        SsalEventType::InitializeCluster((event, _log)) => {
            on_initialize_cluster(event, context).await
        }
        SsalEventType::ContractError(error) => {
            tracing::warn!("{:?}", error)
        }
        _ => {}
    }
}

async fn on_new_block(block: Block<H256>, context: ()) {
    let block_number = block.number.unwrap();
    tracing::info!("{}", block_number);
}

async fn on_initialize_cluster(event: InitializeClusterEventFilter, context: ()) {
    tracing::info!("Cluster ID: {:?}", event.cluster_id)
}
