use ssal::avs::{
    types::{Avs::NewTaskCreated, Block, SsalEventType},
    SsalEventListener,
};
use tokio::time::{sleep, Duration};

use crate::{state::AppState, task::helper::TaskExt, types::SequencerList};

pub fn init(context: AppState) {
    tokio::spawn(async move {
        loop {
            let ssal_event_listener = SsalEventListener::connect(
                context.config().ethereum_websocket_url(),
                context.config().ssal_contract_address(),
            )
            .await
            .map_task();

            if let Some(ssal_event_listener) = ssal_event_listener {
                ssal_event_listener
                    .init(event_callback, context.clone())
                    .await
                    .map_task();
            }

            sleep(Duration::from_secs(3)).await;
            tracing::warn!("Reconnecting the event listener..");
        }
    });
}

async fn event_callback(event_type: SsalEventType, context: AppState) {
    match event_type {
        SsalEventType::NewBlock(block) => on_new_block(block, context.clone()).await,
        SsalEventType::BlockCommitment((block_commitment, _log)) => {
            on_block_commitment(block_commitment, context.clone()).await;
        }
        _ => {}
    }
}

async fn on_new_block(block: Block, context: AppState) {
    if let Some(block_number) = block.header.number {
        let sequencer_list = context
            .ssal_client()
            .get_sequencer_list(context.config().cluster_id())
            .await
            .map_task();

        if let Some(sequencer_list) = sequencer_list {
            SequencerList::from(sequencer_list)
                .put(&context.database(), block_number)
                .map_task();
        }
    }
}

async fn on_block_commitment(block_commitment: BlockCommitmentEvent, context: AppState) {
    let cluster_id = block_commitment.clusterId.to_string();
    if &cluster_id == context.config().cluster_id() {
        // TODO: Vote.
    }
}
