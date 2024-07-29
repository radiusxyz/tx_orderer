use database::Database;
use json_rpc::RpcServer;
use seeder::{
    cli::{Cli, Commands, ConfigOption, ConfigPath},
    error::Error,
    rpc::*,
    task::event_listener,
};
use ssal::avs::seeder::rpc::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();

    let mut cli = Cli::init();

    match cli.command {
        Commands::Init { ref config_path } => ConfigPath::init(config_path)?,
        Commands::Start {
            ref mut config_option,
        } => {
            let config = ConfigOption::load_config(config_option)?;

            let config_path = config.path.as_ref().unwrap();
            let seeder_rpc_endpoint = config.seeder_rpc_endpoint.as_ref().unwrap();

            let provider_rpc_endpoint = config.provider_rpc_endpoint.as_ref().unwrap();
            let contract_address = config.contract_address.as_ref().unwrap();

            // Initialize a local database.
            Database::new(config_path.join("database"))?.init();

            event_listener::init(provider_rpc_endpoint, contract_address);

            let rpc_server_handle = RpcServer::new(())
                .register_rpc_method(Register::METHOD_NAME, register::handler)?
                .register_rpc_method(
                    GetSequencerRpcUrlList::METHOD_NAME,
                    get_sequencer_url_list::handler,
                )?
                .init(seeder_rpc_endpoint)
                .await?;

            tracing::info!("Seeder server starting at {}", seeder_rpc_endpoint);
            rpc_server_handle.stopped().await;
        }
    }

    Ok(())
}
