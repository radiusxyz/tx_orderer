use std::{env, io::stdin};

use database::Database;
use sequencer_avs::{config::Config, error::Error, state::AppState, task::event_manager};
use ssal::avs::SsalClient;

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

    // Initialize the database.
    Database::new(config.database_path())?.init();
    tracing::info!(
        "Succesfully initialized the database at {:?}.",
        config.database_path(),
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

    // Initialize an application-wide state instance.
    let app_state = AppState::new(config, ssal_client, None);

    // Initialize the SSAL event manager.
    event_manager::init(app_state.clone());
    tracing::info!("Successfully initialized the event listener.");

    let mut rollup_address = String::new();
    stdin().read_line(&mut rollup_address).unwrap();
    app_state
        .ssal_client()
        .initialize_cluster(rollup_address)
        .await?;

    loop {}
}
