use radius_sequencer_sdk::{json_rpc::RpcServer, kvstore::KvStore as Database};
use sequencer::{
    cli::{Cli, Commands, Config, ConfigPath},
    client::SeederClient,
    error::Error,
    models::{ClusterIdListModel, SequencingInfoModel},
    rpc::{cluster, external, internal},
    state::AppState,
    task::radius_liveness_event_listener,
    types::{PlatForm, ServiceType},
};
use tokio::task::JoinHandle;
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
            // Load the configuration from the path
            let config = Config::load(config_option)?;

            tracing::info!(
                "Successfully loaded the configuration file at {:?}.",
                config.path(),
            );

            // Initialize the database
            Database::new(config.database_path())?.init();
            tracing::info!(
                "Successfully initialized the database at {:?}.",
                config.database_path(),
            );

            let sequencing_info_model = SequencingInfoModel::get()?;

            // Add listener for each sequencing info
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

            // Initialize an application-wide state instance
            let mut app_state = AppState::new(config.clone());

            // Initialize the internal RPC server
            initialize_internal_rpc_server(&app_state).await?;

            // Initialize the cluster RPC server
            initialize_cluster_rpc_server(&app_state).await?;

            // Initialize clusters
            // initialize_cluster(&mut app_state).await?;

            // Initialize the external RPC server.
            let server_handle = initialize_external_rpc_server(&app_state).await?;

            server_handle.await.unwrap();
        }
    }

    Ok(())
}

async fn register_sequencer_rpc_url(
    seeder_client: &SeederClient,
    sequencer_address: &str,
    sequencer_rpc_url: &str,
) -> Result<(), Error> {
    seeder_client
        .register_sequencer_rpc_url(
            sequencer_address.to_string().into(),
            sequencer_rpc_url.to_string(),
        )
        .await
}

// async fn initialize_cluster(app_state: &mut AppState) -> Result<(), Error> {
//     let cluster_id_list = ClusterIdListModel::get()?.rollup_id_list().clone();
//     tracing::info!("Current rollup ids {:?}.", cluster_id_list);

//     if cluster_id_list.is_empty() {
//         return Ok(());
//     }

//     let config = app_state.config();
//     let signing_key = config.signing_key();
//     // TODO: change
//     let sequencer_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

//     // Initialize seeder client
//     let seeder_rpc_url = config.seeder_rpc_url().to_string();
//     let seeder_client = SeederClient::new(seeder_rpc_url)?;

//     // The cluster rpc url is the rpc url of the sequencer
//     let cluster_rpc_url = config.cluster_rpc_url().to_string();

//     // Register sequencer rpc url (with cluster rpc url) to seeder
//     tracing::info!(
//         "Registering sequencer rpc url: {:?} {:?}",
//         sequencer_address,
//         cluster_rpc_url
//     );

//     // let _ = register_sequencer_rpc_url(&seeder_client, sequencer_address, &cluster_rpc_url).await?;

//     for cluster_id in cluster_id_list.iter() {
//         let mut rollup_cluster: RollupCluster = RollupCluster::new(cluster_id.clone());
//         let rollup_id = cluster_id.clone();

//         let rollup = RollupModel::get(&rollup_id.clone())?;
//         let rollup = rollup.rollup();

//         // Get sequencer rpc urls from seeder
//         let rollup_sequencer_rpc_urls = seeder_client.get_sequencer_rpc_urls(&rollup_id).await?;

//         let rollup_sequencer_address_list = rollup_sequencer_rpc_urls
//             .keys()
//             .cloned()
//             .collect::<Vec<Address>>();

//         let mut sequencer_rpc_clients = HashMap::new();
//         for rollup_sequencer_address in rollup_sequencer_address_list.iter() {
//             let rpc_client =
//                 RpcClient::new(rollup_sequencer_rpc_urls[rollup_sequencer_address].clone())?;
//             sequencer_rpc_clients.insert(rollup_sequencer_address.clone(), rpc_client);
//         }

//         // Leader selection
//         // let leader_sequencer_index =
//         // rollup.block_height() % rollup_sequencer_address_list.len() as u64;
//         let leader_sequencer_index = 0;
//         let leader_sequencer_address =
//             rollup_sequencer_address_list[leader_sequencer_index as usize].clone();

//         let _ = rollup_cluster
//             .set_sequencer_rpc_clients(sequencer_rpc_clients)
//             .await;
//         let _ = rollup_cluster
//             .set_leader_address(leader_sequencer_address)
//             .await;

//         let _ = app_state
//             .set_rollup_cluster(&rollup_id, rollup_cluster)
//             .await;
//     }

//     Ok(())
// }

async fn initialize_internal_rpc_server(app_state: &AppState) -> Result<(), Error> {
    let internal_rpc_url = app_state.config().internal_rpc_url().to_string();

    // Initialize the internal RPC server.
    let internal_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(
            internal::Deregister::METHOD_NAME,
            internal::Deregister::handler,
        )?
        .register_rpc_method(
            internal::AddSequencingInfo::METHOD_NAME,
            internal::AddSequencingInfo::handler,
        )?
        .register_rpc_method(
            internal::GetSequencingInfo::METHOD_NAME,
            internal::GetSequencingInfo::handler,
        )?
        .register_rpc_method(
            internal::GetSequencingInfos::METHOD_NAME,
            internal::GetSequencingInfos::handler,
        )?
        .register_rpc_method(
            internal::RegisterSequencerRpcUrl::METHOD_NAME,
            internal::RegisterSequencerRpcUrl::handler,
        )?
        .register_rpc_method(
            internal::AddRollup::METHOD_NAME,
            internal::AddRollup::handler,
        )?
        .register_rpc_method(
            internal::GetRollup::METHOD_NAME,
            internal::GetRollup::handler,
        )?
        .register_rpc_method(
            internal::AddCluster::METHOD_NAME,
            internal::AddCluster::handler,
        )?
        .register_rpc_method(
            internal::GetCluster::METHOD_NAME,
            internal::GetCluster::handler,
        )?
        .register_rpc_method(
            internal::GetClusterIdList::METHOD_NAME,
            internal::GetClusterIdList::handler,
        )?
        .init(app_state.config().internal_rpc_url().to_string())
        .await?;

    tracing::info!(
        "Successfully started the internal RPC server: {}",
        internal_rpc_url
    );

    tokio::spawn(async move {
        internal_rpc_server.stopped().await;
    });

    Ok(())
}

async fn initialize_cluster_rpc_server(app_state: &AppState) -> Result<(), Error> {
    let cluster_rpc_url = app_state.config().cluster_rpc_url().to_string();

    let sequencer_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(cluster::SyncBlock::METHOD_NAME, cluster::SyncBlock::handler)?
        .register_rpc_method(
            cluster::SyncTransaction::METHOD_NAME,
            cluster::SyncTransaction::handler,
        )?
        .init(cluster_rpc_url.clone())
        .await?;

    tracing::info!(
        "Successfully started the cluster RPC server: {}",
        cluster_rpc_url
    );

    tokio::spawn(async move {
        sequencer_rpc_server.stopped().await;
    });

    Ok(())
}

async fn initialize_external_rpc_server(app_state: &AppState) -> Result<JoinHandle<()>, Error> {
    let sequencer_rpc_url = app_state.config().sequencer_rpc_url().to_string();

    // Initialize the external RPC server.
    let external_rpc_server = RpcServer::new(app_state.clone())
        .register_rpc_method(external::GetBlock::METHOD_NAME, external::GetBlock::handler)?
        .register_rpc_method(
            external::SendEncryptedTransaction::METHOD_NAME,
            external::SendEncryptedTransaction::handler,
        )?
        .init(sequencer_rpc_url.clone())
        .await?;

    tracing::info!(
        "Successfully started the sequencer RPC server: {}",
        sequencer_rpc_url
    );

    let server_handle = tokio::spawn(async move {
        external_rpc_server.stopped().await;
    });

    Ok(server_handle)
}
