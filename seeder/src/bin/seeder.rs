use database::Database;
use get_sequencer_rpc_url_list::GetSequencerRpcUrlList;
use radius_sequencer_sdk::json_rpc::RpcServer;
use register_sequencer_rpc_url::RegisterSequencerRpcUrl;
use seeder::{
    cli::{Cli, Commands, Config, ConfigOption, ConfigPath, DATABASE_DIR_NAME},
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
            let config = Config::load(config_option)?;

            let seeder_rpc_url = config.seeder_rpc_url();

            event_listener::init(
                config.provider_websocket_url().to_string(),
                config.contract_address().to_string(),
            );

            // Initialize a local database.
            Database::new(config.path().join(DATABASE_DIR_NAME))?.init();

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
