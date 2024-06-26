use std::env;

use database::Database;
use json_rpc::RpcServer;
use sequencer_avs::{
    config::Config,
    error::Error,
    rpc::{external::*, internal::*},
    state::AppState,
    task::event_manager,
    types::*,
};
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
    let keystore_password = arguments
        .get(1)
        .expect("Provide the keystore password.")
        .to_owned();

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

    // Check if the sequencer has failed previously.
    match ClusterMetadata::get_mut().ok() {
        Some(mut cluster_metadata) => {
            tracing::warn!("Found a saved cluster metadata. Recovering the previous state..");

            let ssal_block_number = cluster_metadata.ssal_block_number;
            let rollup_block_number = cluster_metadata.rollup_block_number;

            let cluster = cluster_metadata
                .update(
                    app_state.ssal_client().address(),
                    app_state.config().cluster_id(),
                    ssal_block_number,
                    rollup_block_number,
                )
                .await?;
            app_state.update_cluster(cluster).await;

            // TODO:
            // Check if the `build_block` request had been sent by the rollup before the leader recovered.

            tracing::info!("Succesfully recovered the previous state.");
        }
        None => tracing::info!("Initializing the sequencer.."),
    }

    // Initialize the SSAL event manager.
    event_manager::init(app_state.clone());

    // Initialize JSON-RPC server.
    let external_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(BuildBlock::METHOD_NAME, BuildBlock::handler)?
        .register_rpc_method(SendTransaction::METHOD_NAME, SendTransaction::handler)?
        .init(format!("0.0.0.0:{}", app_state.config().external_port()))
        .await?;

    let internal_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(SyncBuildBlock::METHOD_NAME, SyncBuildBlock::handler)?
        .register_rpc_method(SyncTransaction::METHOD_NAME, SyncTransaction::handler)?
        .register_rpc_method(GetBlock::METHOD_NAME, GetBlock::handler)?
        .register_rpc_method(GetTransaction::METHOD_NAME, GetTransaction::handler)?
        .init(format!("0.0.0.0:{}", app_state.config().internal_port()))
        .await?;

    external_rpc_server.stopped().await;
    internal_rpc_server.stopped().await;

    Ok(())
}
