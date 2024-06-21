use std::{env, io};

use database::Database;
use json_rpc::RpcServer;
use sequencer_avs::{
    config::Config, error::Error, rpc::external::*, state::AppState, task::event_manager,
};
use ssal::avs::SsalClient;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

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
    let config = Config::load(config_path)?;

    // Initialize the database.
    Database::new(config.database_path())?.init();

    // Initialize the SSAL client.
    let ssal_client = SsalClient::new(
        config.ethereum_rpc_url(),
        config.keystore_path(),
        keystore_password,
        config.ssal_contract_address(),
        config.seeder_rpc_address(),
    )?;

    // Initialize an application-wide state instance.
    let app_state = AppState::new(config, ssal_client);

    // Initialize the event manager.
    event_manager::init(app_state.clone());

    // Initialize JSON-RPC server.
    let rpc_server_handle =
        RpcServer::new(app_state.clone()) // RpcServer context is a redundant `Arc` wrapping over `AppState`, but it's okay for now.
            .register_rpc_method(BuildBlock::METHOD_NAME, BuildBlock::handler)?
            .register_rpc_method(SyncBuildBlock::METHOD_NAME, SyncBuildBlock::handler)?
            .register_rpc_method(GetBlock::METHOD_NAME, GetBlock::handler)?
            .register_rpc_method(SendTransaction::METHOD_NAME, SendTransaction::handler)?
            .register_rpc_method(SyncTransaction::METHOD_NAME, SyncTransaction::handler)?
            .init("0.0.0.0:7234")
            .await?;

    rpc_server_handle.stopped().await;

    Ok(())
}
