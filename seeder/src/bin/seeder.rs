use get_sequencer_rpc_urls::GetSequencerRpcUrls;
use radius_sequencer_sdk::{json_rpc::RpcServer, kvstore::KvStore as Database};
use register_sequencer_rpc_url::RegisterSequencerRpcUrl;
use seeder::{
    cli::{Cli, Commands, Config, ConfigOption, ConfigPath, DATABASE_DIR_NAME},
    error::Error,
    models::LivenessModel,
    rpc::*,
    task::event_listener,
};
use sequencer::types::ClusterType;
use tracing::info;

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

            // Initialize a local database.
            Database::new(config.path().join(DATABASE_DIR_NAME))?.init();

            let liveness_model = LivenessModel::get()?;

            liveness_model
                .liveness_info_list
                .iter()
                .for_each(|liveness_info| match liveness_info.cluster_type {
                    ClusterType::Local => {
                        info!(
                            "Init local liveness - provider_websocket_url: {:?}",
                            liveness_info.provider_websocket_url
                        );
                    }
                    ClusterType::EigenLayer => {
                        info!(
                            "Init eigen layer liveness - provider_websocket_url: {:?}",
                            liveness_info.provider_websocket_url
                        );
                        let liveness_contract_address =
                            liveness_info.liveness_contract_address.clone().unwrap();

                        event_listener::init(
                            liveness_info.provider_websocket_url.to_string(),
                            liveness_contract_address.to_string(),
                        );
                    }
                });

            let rpc_server_handle = RpcServer::new(())
                .register_rpc_method(
                    AddSupportLiveness::METHOD_NAME,
                    add_support_liveness::handler,
                )?
                .register_rpc_method(
                    RegisterSequencerRpcUrl::METHOD_NAME,
                    register_sequencer_rpc_url::handler,
                )?
                .register_rpc_method(
                    GetSequencerRpcUrls::METHOD_NAME,
                    get_sequencer_rpc_urls::handler,
                )?
                .init(seeder_rpc_url)
                .await?;

            info!("Seeder server starting at {}", seeder_rpc_url);
            rpc_server_handle.stopped().await;
        }
    }

    Ok(())
}
