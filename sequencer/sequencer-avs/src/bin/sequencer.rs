use std::env;

use database::Database;
use json_rpc::RpcServer;
use sequencer_avs::{
    config::Config, error::Error, rpc::external, task::cluster_manager, types::Me,
};
use ssal::ethereum::SsalClient;

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

    // Initialize the SSAL client.
    let ssal_client = SsalClient::new(
        &config.ssal_rpc_address,
        &config.ssal_private_key,
        &config.contract_address,
        config.cluster_id,
        &config.seeder_rpc_address,
    )
    .await?;

    // Initialize the cluster manager
    cluster_manager::init(&ssal_client);

    // Initialize JSON-RPC server.
    let rpc_server_handle = RpcServer::new()
        .register_rpc_method::<external::BuildBlock>()?
        .register_rpc_method::<external::SyncBuildBlock>()?
        .register_rpc_method::<external::GetBlock>()?
        .register_rpc_method::<external::SendTransaction>()?
        .register_rpc_method::<external::SyncTransaction>()?
        .init(&config.sequencer_rpc_address)
        .await?;

    tokio::spawn(async move {
        rpc_server_handle.stopped().await;
    });

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            // "1" => ssal_client.initialize_cluster(),
            // "2" => ssal_client.register_sequencer(),
            // "3" => ssal_client.derregister_sequencer(),
            _ => continue,
        }
    }
}
