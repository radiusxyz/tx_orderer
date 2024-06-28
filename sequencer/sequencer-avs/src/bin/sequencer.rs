use std::env;

use database::Database;
use json_rpc::RpcServer;
use sequencer_avs::{
    config::Config,
    error::Error,
    rpc::{cluster, external, internal},
    state::AppState,
    task::event_manager,
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

    // Check if the sequencer has failed previously.
    check_failover(&app_state).await?;

    // Initialize the SSAL event manager.
    event_manager::init(app_state.clone());
    tracing::info!("Successfully initialized the event listener.");

    // Initialize the internal RPC server.
    initialize_internal_rpc_server(&app_state).await?;

    // Initialize the cluster RPC server.
    let server_handle = initialize_cluster_rpc_server(&app_state).await?;

    // Initialize the sequencer registration for both EigenLayer and SSAL.
    register_as_operator(&app_state).await?;
    register_sequencer(&app_state).await?;

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

async fn check_failover(app_state: &AppState) -> Result<(), Error> {
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

    Ok(())
}

async fn register_as_operator(app_state: &AppState) -> Result<(), Error> {
    match app_state.ssal_client().is_operator().await? {
        true => {
            tracing::info!(
                "Already registered as an operator. Skipping the operator registration.."
            );
        }
        false => {
            app_state.ssal_client().register_as_operator().await?;
            tracing::info!("Successfully registered as an operator.");
        }
    }

    Ok(())
}

async fn register_sequencer(app_state: &AppState) -> Result<(), Error> {
    match app_state
        .ssal_client()
        .is_registered(
            app_state.config().cluster_id(),
            app_state.ssal_client().address(),
        )
        .await?
    {
        true => {
            tracing::info!("Already registered on the SSAL contract. Skipping the registration..")
        }
        false => {
            app_state
                .ssal_client()
                .register_sequencer(
                    app_state.config().cluster_id(),
                    app_state.config().sequencer_rpc_url(),
                )
                .await?;
            tracing::info!("Successfully registered the sequencer on SSAL contract.");
        }
    }

    Ok(())
}

async fn initialize_cluster_rpc_server(app_state: &AppState) -> Result<JoinHandle<()>, Error> {
    let sequencer_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(
            cluster::BuildBlock::METHOD_NAME,
            cluster::BuildBlock::handler,
        )?
        .register_rpc_method(
            external::SendTransaction::METHOD_NAME,
            external::SendTransaction::handler,
        )?
        .register_rpc_method(
            cluster::SyncBuildBlock::METHOD_NAME,
            cluster::SyncBuildBlock::handler,
        )?
        .register_rpc_method(
            cluster::SyncTransaction::METHOD_NAME,
            cluster::SyncTransaction::handler,
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
