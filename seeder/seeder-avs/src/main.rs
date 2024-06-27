use std::env;

use database::Database;
use json_rpc::RpcServer;
use seeder_avs::{config::Config, error::Error, rpc::*};
use ssal::avs::seeder::rpc::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let arguments: Vec<String> = env::args().skip(1).collect();
    let config_path: String = arguments
        .get(0)
        .expect("Provide the configuration file path.")
        .to_owned();

    let config = Config::load(config_path)?;

    Database::new(config.database_path())?.init();

    let rpc_server_handle = RpcServer::new(())
        .register_rpc_method(Register::METHOD_NAME, register::handler)?
        .register_rpc_method(Deregister::METHOD_NAME, deregister::handler)?
        .register_rpc_method(
            GetSequencerRpcUrlList::METHOD_NAME,
            get_sequencer_url_list::handler,
        )?
        .init(config.seeder_rpc_url())
        .await?;

    tracing::info!("Seeder server starting at {}", config.seeder_rpc_url());
    rpc_server_handle.stopped().await;

    Ok(())
}
