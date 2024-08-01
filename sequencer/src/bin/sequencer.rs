use std::env;

use radius_sequencer_sdk::{json_rpc::RpcServer, kvstore::KvStore as Database};
use sequencer::{
    cli::{Cli, Commands, Config, ConfigOption, ConfigPath, DATABASE_DIR_NAME},
    error::Error,
    rpc::{cluster, external, internal},
    state::AppState,
    task::event_listener,
    types::{ClusterType, RollupCluster},
};
use ssal::avs::LivenessClient;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let mut cli = Cli::init();

    match cli.command {
        Commands::Init { ref config_path } => ConfigPath::init(config_path)?,
        Commands::Start {
            ref mut config_option,
        } => {
            // Load the configuration from the path.
            let config = Config::load(config_option)?;

            tracing::info!(
                "Successfully loaded the configuration file at {:?}.",
                config.path(),
            );

            // Initialize the database.
            Database::new(config.database_path())?.init();
            tracing::info!(
                "Successfully initialized the database at {:?}.",
                config.database_path(),
            );

            // Initialize an application-wide state instance.
            let app_state = AppState::new(config.clone());

            if config.cluster_type() == &ClusterType::EigenLayer {
                event_listener::init(
                    config.liveness_provider_websocket_url().to_string(),
                    config
                        .liveness_contract_address()
                        .as_ref()
                        .unwrap()
                        .to_string(),
                );
            }

            // Initialize the internal RPC server.
            initialize_internal_rpc_server(&app_state).await?;

            // Initialize the cluster RPC server.
            initialize_cluster_rpc_server(&app_state).await?;

            // Initialize the external RPC server.
            let server_handle = initialize_external_rpc_server(&app_state).await?;

            server_handle.await.unwrap();
        }
    }

    Ok(())
}

async fn initialize_internal_rpc_server(app_state: &AppState) -> Result<(), Error> {
    let internal_rpc_url = app_state.config().internal_rpc_url().to_string();

    // Initialize the internal RPC server.
    let internal_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(
            internal::Deregister::METHOD_NAME,
            internal::Deregister::handler,
        )?
        .init(app_state.config().internal_rpc_url().to_string())
        .await?;

    tracing::info!(
        "Successfully started the internal RPC server: {}",
        internal_rpc_url
    );

    tokio::spawn(async move {
        internal_rpc_server.stopped().await;
    });

    Ok(())
}

async fn initialize_cluster_rpc_server(app_state: &AppState) -> Result<(), Error> {
    let cluster_rpc_url = app_state.config().cluster_rpc_url().to_string();

    let sequencer_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(cluster::SyncBlock::METHOD_NAME, cluster::SyncBlock::handler)?
        .register_rpc_method(
            cluster::SyncTransaction::METHOD_NAME,
            cluster::SyncTransaction::handler,
        )?
        .init(cluster_rpc_url.clone())
        .await?;

    tracing::info!(
        "Successfully started the cluster RPC server: {}",
        cluster_rpc_url
    );

    tokio::spawn(async move {
        sequencer_rpc_server.stopped().await;
    });

    Ok(())
}

async fn initialize_external_rpc_server(app_state: &AppState) -> Result<JoinHandle<()>, Error> {
    let sequencer_rpc_url = app_state.config().sequencer_rpc_url().to_string();

    // Initialize the external RPC server.
    let external_rpc_server = RpcServer::new(app_state.clone())
        // .register_rpc_method(
        //     external::SendTransaction::METHOD_NAME,
        //     external::SendTransaction::handler,
        // )?
        .init(sequencer_rpc_url.clone())
        .await?;

    tracing::info!(
        "Successfully started the sequencer RPC server: {}",
        sequencer_rpc_url
    );

    let server_handle = tokio::spawn(async move {
        external_rpc_server.stopped().await;
    });

    Ok(server_handle)
}
