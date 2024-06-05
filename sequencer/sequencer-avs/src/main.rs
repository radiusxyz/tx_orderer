use std::env;

use database::{database, Database};
use json_rpc::RpcServer;
use sequencer_avs::{
    config::Config, error::Error, rpc::external, task::cluster_manager, types::Me,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path = arguments
        .get(0)
        .expect("Provide the config file path.")
        .to_owned();

    // Load the configuration from the path.
    let config = Config::load(config_path)?;

    // Initialize the database as a global singleton called by `database::database()`.
    Database::new(&config.database_path)?.init();

    // Store my public key.
    Me::try_from(config.sequencer_public_key.as_str())?.put()?;

    // Initialize JSON-RPC server.
    RpcServer::new()
        .register_rpc_method::<external::CloseBlock>()?
        .register_rpc_method::<external::SyncCloseBlock>()?
        .register_rpc_method::<external::SendTransaction>()?
        .register_rpc_method::<external::SyncTransaction>()?
        .init(&config.sequencer_rpc_address)
        .await?;

    // Initialize the cluster manager.
    cluster_manager::init(&config)?;

    Ok(())
}
