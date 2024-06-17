use std::{env, io};

use database::Database;
use json_rpc::RpcServer;
use sequencer_avs::{config::Config, error::Error, rpc::external::*, types::Me};
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

    // Store my public key.
    Me::from(ssal_client.public_key()).put()?;

    // Initialize the cluster manager
    // cluster_manager::init(&ssal_client);

    // Initialize JSON-RPC server.
    let rpc_server_handle = RpcServer::new(ssal_client.clone())
        .register_rpc_method(BuildBlock::METHOD_NAME, BuildBlock::handler)?
        .register_rpc_method(SyncBuildBlock::METHOD_NAME, SyncBuildBlock::handler)?
        .register_rpc_method(GetBlock::METHOD_NAME, GetBlock::handler)?
        .register_rpc_method(SendTransaction::METHOD_NAME, SendTransaction::handler)?
        .register_rpc_method(SyncTransaction::METHOD_NAME, SyncTransaction::handler)?
        .init(&config.sequencer_rpc_address)
        .await?;

    tokio::spawn(async move {
        rpc_server_handle.stopped().await;
    });

    loop {
        println!("1. Initialize a cluster\n2. Register the sequencer\n3. Deregister the sequencer");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => initialize(&config, &ssal_client).await,
            "2" => register(&config, &ssal_client).await,
            "3" => deregister(&ssal_client).await,
            _ => continue,
        }
    }
}

async fn initialize(config: &Config, ssal_client: &SsalClient) {
    println!("Rollup Public Key:");
    let mut rollup_public_key = String::new();
    io::stdin().read_line(&mut rollup_public_key).unwrap();

    match ssal_client
        .initialize_cluster(&config.sequencer_rpc_address, rollup_public_key.trim())
        .await
    {
        Ok(_) => (),
        Err(error) => tracing::error!("{}", error),
    }
}

async fn register(config: &Config, ssal_client: &SsalClient) {
    match ssal_client
        .register_sequencer(&config.sequencer_rpc_address)
        .await
    {
        Ok(_) => (),
        Err(error) => tracing::error!("{}", error),
    }
}

async fn deregister(ssal_client: &SsalClient) {
    match ssal_client.deregister_sequencer().await {
        Ok(_) => (),
        Err(error) => tracing::error!("{}", error),
    }
}
