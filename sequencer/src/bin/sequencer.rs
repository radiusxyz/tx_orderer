use std::collections::HashMap;

use ethers::abi::Hash;
use radius_sequencer_sdk::{
    json_rpc::{RpcClient, RpcServer},
    kvstore::KvStore as Database,
    liveness::publisher::Publisher,
};
use sequencer::{
    cli::{Cli, Commands, Config, ConfigPath},
    client::SeederClient,
    error::Error,
    models::{
        ClusterIdListModel, RollupIdListModel, RollupMetadataModel, RollupModel,
        SequencingInfoModel,
    },
    rpc::{cluster, external, internal},
    state::AppState,
    task::radius_liveness_event_listener,
    types::{
        ClusterId, PlatForm, RollupId, RollupMetadata, SequencingFunctionType, ServiceType,
        SigningKey,
    },
    util::initialize_liveness_cluster,
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
            let sequencing_infos = sequencing_info_model.sequencing_infos();

            // Add listener for each sequencing info
            sequencing_infos.iter().for_each(
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

            // Initialize seeder client
            let seeder_rpc_url = config.seeder_rpc_url();
            let seeder_client = SeederClient::new(seeder_rpc_url)?;
            tracing::info!(
                "Successfully initialized seeder client {:?}.",
                seeder_rpc_url,
            );

            let rollup_id_list_model = RollupIdListModel::get()?;
            let rollup_id_list = rollup_id_list_model.rollup_id_list();

            let mut rollup_metadatas: HashMap<RollupId, RollupMetadata> = HashMap::new();
            let mut rollup_cluster_ids: HashMap<RollupId, ClusterId> = HashMap::new();

            rollup_id_list.iter().for_each(|rollup_id| {
                let rollup_model = RollupModel::get(rollup_id).unwrap();
                let rollup_metadata_model = RollupMetadataModel::get(rollup_id).unwrap();

                let cluster_id = rollup_model.cluster_id().clone();
                let rollup_metadata = rollup_metadata_model.rollup_metadata().clone();

                rollup_metadatas.insert(rollup_id.clone(), rollup_metadata);
                rollup_cluster_ids.insert(rollup_id.clone(), cluster_id);
            });

            // Initialize an application-wide state instance
            let app_state = AppState::new(
                config.clone(),
                rollup_metadatas,
                rollup_cluster_ids, // rollup_cluster_ids,
                sequencing_infos.clone(),
                seeder_client,
            );

            // Initialize the internal RPC server
            initialize_internal_rpc_server(&app_state).await?;

            // Initialize the cluster RPC server
            initialize_cluster_rpc_server(&app_state).await?;

            // Initialize clusters
            initialize_clusters(&app_state).await?;

            // Initialize the external RPC server.
            let server_handle = initialize_external_rpc_server(&app_state).await?;

            server_handle.await.unwrap();
        }
    }

    Ok(())
}

async fn initialize_clusters(app_state: &AppState) -> Result<(), Error> {
    let config = app_state.config();
    let seeder_client = app_state.seeder_client();
    let signing_key = config.signing_key();

    let address = config.address();

    // The cluster rpc url is the rpc url of the sequencer
    let cluster_rpc_url = config.cluster_rpc_url().to_string();

    // Register sequencer rpc url (with cluster rpc url) to seeder
    tracing::info!("Registering rpc url: {:?} {:?}", address, cluster_rpc_url);
    let _ = seeder_client
        .register_rpc_url(address, cluster_rpc_url.to_string())
        .await?;

    let sequencing_info_model = SequencingInfoModel::get()?;
    let sequencing_infos = sequencing_info_model.sequencing_infos();

    for (sequencing_info_key, sequencing_info) in sequencing_infos.iter() {
        info!(
            "platform: {:?}, sequencing_function_type: {:?}, service_type: {:?}",
            sequencing_info_key.platform(),
            sequencing_info_key.sequencing_function_type(),
            sequencing_info_key.service_type()
        );

        // Get all cluster ids for each sequencing info
        let cluster_id_list_model = ClusterIdListModel::get(
            sequencing_info_key.platform(),
            sequencing_info_key.sequencing_function_type(),
            sequencing_info_key.service_type(),
        )
        .unwrap();

        // Initialize each cluster
        for cluster_id in cluster_id_list_model.cluster_id_list.iter() {
            info!("initialize_cluster: {:?}", cluster_id);

            match sequencing_info_key.sequencing_function_type() {
                SequencingFunctionType::Liveness => {
                    let cluster = initialize_liveness_cluster(
                        &SigningKey::from(signing_key.clone()),
                        &seeder_client,
                        &sequencing_info_key,
                        &sequencing_info,
                        &cluster_id,
                    )
                    .await?;

                    app_state.set_cluster(cluster).await;
                }
                SequencingFunctionType::Validation => {
                    // TODO:
                }
            }
        }
    }
    Ok(())
}

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
            internal::RegisterRpcUrl::METHOD_NAME,
            internal::RegisterRpcUrl::handler,
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
        .register_rpc_method(
            internal::GetContext::METHOD_NAME,
            internal::GetContext::handler,
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
