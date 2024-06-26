use std::{env, time::Duration};

use database::{database, Database};
use json_rpc::RpcClient;
use sequencer_avs::{
    config::Config, error::Error, rpc::external::*, state::AppState, task::TraceExt, types::*,
};
use ssal::avs::{types::*, SsalClient, SsalEventListener};
use tokio::time::sleep;

const SSAL_BLOCK_NUMBER_KEY: &'static str = "0";

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    std::panic::set_hook(Box::new(|panic_info| tracing::error!("{}", panic_info)));

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path = arguments
        .get(0)
        .expect("Provide the config file path.")
        .to_owned();
    let keystore_password = arguments
        .get(1)
        .expect("Provide the keystore password.")
        .to_owned();
    let block_margin: u64 = arguments
        .get(2)
        .expect("Provide the block margin.")
        .parse()
        .expect("Failed to parse the block margin argument to `u64`");
    let block_creation_time: u64 = arguments
        .get(3)
        .expect("Provide the block creation time.")
        .parse()
        .expect("Failed to parse the block creation time argument to `u64`");

    // Load the configuration from the path.
    let config = Config::load(&config_path)?;
    tracing::info!(
        "Successfully loaded the configuration file at {}",
        config_path,
    );

    // Initialize the database.
    Database::new(config.database_path())?.init();
    tracing::info!(
        "Succesfully initialized the database at {:?}",
        config.database_path(),
    );

    // Initialize the SSAL client.
    let ssal_client = SsalClient::new(
        config.ethereum_rpc_url(),
        config.keystore_path(),
        keystore_password,
        config.seeder_rpc_url(),
        config.ssal_contract_address(),
        config.delegation_manager_contract_address(),
        config.stake_registry_contract_address(),
        config.avs_directory_contract_address(),
        config.avs_contract_address(),
    )?;
    tracing::info!("Successfully initialized the SSAL client");

    // Initialize an application-wide state instance.
    let app_state = AppState::new(config, ssal_client, None);

    // Initialize the event manager.
    event_manager(app_state.clone());

    // Load the database instance.
    let database = database()?;

    let previous_ssal_block_number = 0;
    let mut rollup_block_number: u64 = 0;
    loop {
        let current_ssal_block_number = database
            .get::<&'static str, u64>(&SSAL_BLOCK_NUMBER_KEY)
            .ok_or_trace();

        if let Some(ssal_block_number) = current_ssal_block_number {
            let sequencer_list = SequencerList::get(ssal_block_number - block_margin).ok_or_trace();

            if let Some(sequencer_list) = sequencer_list {
                let leader_index = rollup_block_number
                    .checked_rem(sequencer_list.len() as u64)
                    .ok_or(Error::EmptySequencerList)
                    .ok_or_trace();
            }
        }

        sleep(Duration::from_secs(block_creation_time)).await;
    }
}

fn event_manager(context: AppState) {
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
        SsalEventType::NewBlock(block) => {
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

                database()
                    .unwrap()
                    .put(&SSAL_BLOCK_NUMBER_KEY, &block_number)
                    .unwrap();
            }
        }
        SsalEventType::BlockCommitment((event, _log)) => {
            if &event.clusterID.to_string() == context.config().cluster_id() {
                request_block(event.blockNumber).await.ok_or_trace();
            }
        }
        _ => {}
    }
}

async fn request_block(rollup_block_number: u64) -> Result<(), Error> {
    let sequencer_list = SequencerList::get(rollup_block_number).ok_or_trace();

    if let Some(sequencer_list) = sequencer_list {}

    Ok(())
}

async fn request_build_block(block_margin: u64, rollup_block_number: u64) -> Result<(), Error> {
    let ssal_block_number =
        database()?.get::<&'static str, u64>(&SSAL_BLOCK_NUMBER_KEY)? - block_margin;
    let sequencer_list = SequencerList::get(ssal_block_number)?;

    let rpc_method = BuildBlock {
        ssal_block_number,
        rollup_block_number,
    };

    Ok(())
}
