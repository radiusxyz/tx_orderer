use std::env;

use database::Database;
use json_rpc::RpcServer;
use sequencer::{
    config::Config,
    error::Error,
    rpc::{cluster, external, internal},
    state::AppState,
    types::*,
};
use ssal::avs::SsalClient;
use tokio::task::JoinHandle;

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
        "Successfully initialized the database at {:?}.",
        config.database_path(),
    );

    // Initialize the SSAL client. (TODO: remove)
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

    // // Check if the sequencer has failed previously.
    // check_failover(&app_state).await?;

    // Initialize the SSAL event manager.
    // event_manager::init(app_state.clone());
    // tracing::info!("Successfully initialized the event listener.");

    // Initialize the internal RPC server.
    initialize_internal_rpc_server(&app_state).await?;

    // Initialize the external RPC server.
    initialize_external_rpc_server(&app_state).await?;

    // Initialize the cluster RPC server.
    let server_handle = initialize_cluster_rpc_server(&app_state).await?;

    // Initialize the sequencer registration for both EigenLayer and SSAL.
    // register_as_operator(&app_state).await?;
    // register_on_avs(&app_state).await?;
    // register_sequencer(&app_state).await?;

    server_handle.await.unwrap();

    Ok(())
}

async fn initialize_internal_rpc_server(app_state: &AppState) -> Result<(), Error> {
    // Initialize the internal RPC server.
    let internal_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(
            internal::Deregister::METHOD_NAME,
            internal::Deregister::handler,
        )?
        .init("127.0.0.1:7234")
        .await?;

    tokio::spawn(async move {
        internal_rpc_server.stopped().await;
    });

    Ok(())
}

async fn initialize_external_rpc_server(app_state: &AppState) -> Result<(), Error> {
    // Initialize the external RPC server.
    let external_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(
            external::SendTransaction::METHOD_NAME,
            external::SendTransaction::handler,
        )?
        .init("127.0.0.1:7235")
        .await?;

    tokio::spawn(async move {
        external_rpc_server.stopped().await;
    });

    Ok(())
}

async fn initialize_cluster_rpc_server(app_state: &AppState) -> Result<JoinHandle<()>, Error> {
    let sequencer_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(
            cluster::BuildBlock::METHOD_NAME,
            cluster::BuildBlock::handler,
        )?
        .register_rpc_method(cluster::SyncBlock::METHOD_NAME, cluster::SyncBlock::handler)?
        .register_rpc_method(
            cluster::SyncRequest::METHOD_NAME,
            cluster::SyncRequest::handler,
        )?
        .register_rpc_method(cluster::GetBlock::METHOD_NAME, cluster::GetBlock::handler)?
        .register_rpc_method(
            cluster::GetTransaction::METHOD_NAME,
            cluster::GetTransaction::handler,
        )?
        .init(format!("0.0.0.0:{}", app_state.config().sequencer_port()?))
        .await?;

    tracing::info!(
        "Successfully started the RPC server: {}",
        format!("0.0.0.0:{}", app_state.config().sequencer_port()?)
    );

    let server_handle = tokio::spawn(async move {
        sequencer_rpc_server.stopped().await;
    });

    Ok(server_handle)
}
