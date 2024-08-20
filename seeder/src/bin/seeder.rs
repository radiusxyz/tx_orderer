use radius_sequencer_sdk::{json_rpc::RpcServer, kvstore::KvStore as Database};
use seeder::{
    cli::{Cli, Commands, Config, ConfigPath, DATABASE_DIR_NAME},
    error::Error,
    rpc::*,
    task::radius_liveness_event_listener,
};
use sequencer::{
    models::SequencingInfoModel,
    types::{PlatForm, ServiceType},
};
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

            let sequencing_info_model = SequencingInfoModel::get()?;

            sequencing_info_model.sequencing_infos().iter().for_each(
                |(sequencing_info_key, sequencing_info)| {
                    info!(
                        "platform: {:?}, sequencing_function_type: {:?}, service_type: {:?}",
                        sequencing_info_key.platform(), sequencing_info_key.sequencing_function_type(), sequencing_info_key.service_type()
                    );

                    match sequencing_info_key.platform() {
                        PlatForm::Local => {
                            // TODO:
                            info!("Init local platform (TODO)");
                        }
                        PlatForm::Ethereum => match sequencing_info_key.sequencing_function_type() {
                            sequencer::types::SequencingFunctionType::Liveness => {
                                match sequencing_info_key.service_type() {
                                    ServiceType::Radius => {
                                        info!(
                                            "Init radius liveness - provider_websocket_url: {:?}",
                                            sequencing_info.provider_websocket_url
                                        );

                                        radius_liveness_event_listener::init(
                                            sequencing_info.clone(),
                                        );
                                    }
                                    _ => {
                                        // TODO:
                                        info!(
                                            "Init other liveness (TODO) - provider_websocket_url: {:?}",
                                            sequencing_info.provider_websocket_url
                                        );
                                    }
                                }
                            }
                            sequencer::types::SequencingFunctionType::Validation => {}
                        },
                    }
                },
            );

            let rpc_server_handle = RpcServer::new(())
                .register_rpc_method(AddSequencingInfo::METHOD_NAME, AddSequencingInfo::handler)?
                .register_rpc_method(GetRpcUrl::METHOD_NAME, GetRpcUrl::handler)?
                .register_rpc_method(GetSequencingInfo::METHOD_NAME, GetSequencingInfo::handler)?
                .register_rpc_method(GetSequencingInfos::METHOD_NAME, GetSequencingInfos::handler)?
                .register_rpc_method(InitializeCluster::METHOD_NAME, InitializeCluster::handler)?
                .register_rpc_method(GetCluster::METHOD_NAME, GetCluster::handler)?
                .register_rpc_method(Register::METHOD_NAME, Register::handler)?
                .register_rpc_method(Deregister::METHOD_NAME, Deregister::handler)?
                .register_rpc_method(RegisterRpcUrl::METHOD_NAME, RegisterRpcUrl::handler)?
                .register_rpc_method(GetRpcUrlList::METHOD_NAME, GetRpcUrlList::handler)?
                .register_rpc_method(GetClusterList::METHOD_NAME, GetClusterList::handler)?
                .init(seeder_rpc_url)
                .await?;

            info!("Seeder server starting at {}", seeder_rpc_url);
            rpc_server_handle.stopped().await;
        }
    }

    Ok(())
}
