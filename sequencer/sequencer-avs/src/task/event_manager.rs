use ssal::avs::{
    types::{Avs::NewTaskCreated, Block, SsalEventType},
    SsalEventListener,
};
use tokio::time::{sleep, Duration};

use crate::{
    state::AppState,
    task::TraceExt,
    types::{BlockCommitment, SequencerList},
};

pub fn init(context: AppState) {
    tokio::spawn(async move {
        loop {
            let ssal_event_listener = SsalEventListener::connect(
                context.config().ethereum_websocket_url(),
                context.config().ssal_contract_address(),
                context.config().avs_contract_address(),
            )
            .await
            .ok_or_trace();

            if let Some(ssal_event_listener) = ssal_event_listener {
                ssal_event_listener
                    .init(event_callback, context.clone())
                    .await
                    .ok_or_trace();
            }

            sleep(Duration::from_secs(3)).await;
            tracing::warn!("Reconnecting the event listener..");
        }
    });
}

async fn event_callback(event_type: SsalEventType, context: AppState) {
    match event_type {
        SsalEventType::NewBlock(event) => on_new_block(event, context.clone()).await,
        SsalEventType::BlockCommitment((event, _log)) => {
            on_block_commitment(event, context.clone()).await;
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
            .ok_or_trace();

        if let Some(sequencer_list) = sequencer_list {
            SequencerList::from(sequencer_list)
                .put(block_number)
                .ok_or_trace();
        }
    }
}

async fn on_block_commitment(event: NewTaskCreated, context: AppState) {
    let cluster_id = event.clusterID.to_string();
    if &cluster_id == context.config().cluster_id() {
        let block_commitment = BlockCommitment::get(event.blockNumber).ok_or_trace();

        if let Some(block_commitment) = block_commitment {
            let block_commitment_bytes = block_commitment.to_bytes().ok_or_trace();

            if let Some(block_commitment_bytes) = block_commitment_bytes {
                context
                    .ssal_client()
                    .respond_to_task(event.task, event.taskIndex, block_commitment_bytes)
                    .await
                    .ok_or_trace();
            }
        }
    }
}
