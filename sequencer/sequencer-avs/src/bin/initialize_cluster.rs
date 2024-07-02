use std::{env, io::stdin};

use sequencer_avs::{config::Config, error::Error};
use ssal::avs::{
    types::{Ssal::InitializeClusterEvent, SsalEventType},
    SsalClient, SsalEventListener,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    std::panic::set_hook(Box::new(|panic_info| tracing::error!("{}", panic_info)));

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path = arguments
        .get(0)
        .expect("Provide the config file path.")
        .to_owned();

    // Load the configuration from the path.
    let config = Config::load(&config_path)?;
    tracing::info!(
        "Successfully loaded the configuration file at {}.",
        config_path,
    );

    // Initialize the SSAL client.
    let ssal_client = SsalClient::new(
        config.ethereum_rpc_url(),
        config.signing_key(),
        config.seeder_rpc_url(),
        config.ssal_contract_address(),
        config.delegation_manager_contract_address(),
        config.stake_registry_contract_address(),
        config.avs_directory_contract_address(),
        config.avs_contract_address(),
    )?;
    tracing::info!("Successfully initialized the SSAL client.");

    println!("Rollup Address:");
    let mut rollup_address = String::new();
    stdin().read_line(&mut rollup_address).unwrap();
    ssal_client
        .initialize_cluster(rollup_address.trim())
        .await?;

    let event_listener = SsalEventListener::connect(
        config.ethereum_websocket_url(),
        config.ssal_contract_address(),
        config.avs_contract_address(),
    )
    .await?;

    event_listener.init(callback, (config, config_path)).await?;

    Ok(())
}

async fn callback(event_type: SsalEventType, context: (Config, String)) {
    match event_type {
        SsalEventType::InitializeCluster((event, _log)) => {
            on_initialize_cluster(event, context).await
        }
        _ => {}
    }
}

async fn on_initialize_cluster(event: InitializeClusterEvent, context: (Config, String)) {
    let cluster_id = event.clusterID.to_string();
    tracing::info!("Initialized a new cluster (ID = {})", cluster_id);
    context.0.save(context.1, cluster_id);
    std::process::exit(0);
}
