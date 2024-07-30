use database::Database;
use get_sequencer_rpc_url_list::GetSequencerRpcUrlList;
use json_rpc::RpcServer;
use register_sequencer_rpc_url::RegisterSequencerRpcUrl;
use seeder::{
    cli::{Cli, Commands, ConfigOption, ConfigPath},
    error::Error,
    rpc::*,
    task::event_listener,
};

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
            let seeder_rpc_url = config.seeder_rpc_url.as_ref().unwrap();

            let provider_websocket_url = config.provider_websocket_url.unwrap();
            let contract_address = config.contract_address.unwrap();

            event_listener::init(provider_websocket_url, contract_address);

            // Initialize a local database.
            Database::new(config_path.join("database"))?.init();

            let rpc_server_handle = RpcServer::new(())
                .register_rpc_method(
                    RegisterSequencerRpcUrl::METHOD_NAME,
                    register_sequencer_rpc_url::handler,
                )?
                .register_rpc_method(
                    GetSequencerRpcUrlList::METHOD_NAME,
                    get_sequencer_rpc_url_list::handler,
                )?
                .init(seeder_rpc_url)
                .await?;

            tracing::info!("Seeder server starting at {}", seeder_rpc_url);
            rpc_server_handle.stopped().await;
        }
    }

    Ok(())
}
