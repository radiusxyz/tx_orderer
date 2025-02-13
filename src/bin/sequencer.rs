use clap::{Parser, Subcommand};
use futures::future::try_join_all;
use radius_sdk::{
    json_rpc::{client::RpcClient, server::RpcServer},
    kvstore::{CachedKvStore, KvStoreBuilder},
    util::{get_resource_limit, set_resource_limit, ResourceType},
};
use sequencer::{
    client::{
        distributed_key_generation::DistributedKeyGenerationClient,
        liveness_service_manager::{self},
        seeder::SeederClient,
        validation_service_manager,
    },
    error::{self, Error},
    logger::PanicLog,
    merkle_tree_manager::MerkleTreeManager,
    rpc::{cluster, external, internal},
    state::AppState,
    types::*,
    util::initialize_logger,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Parser, Serialize)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    fn init() -> Self {
        Cli::parse()
    }
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]
enum Commands {
    /// Initializes a node
    Init {
        #[clap(flatten)]
        config_path: ConfigPath,
    },
    /// Starts the node
    Start {
        #[clap(flatten)]
        config_option: ConfigOption,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::panic::set_hook(Box::new(|panic_info| {
        let panic_log: PanicLog = panic_info.into();
        tracing::error!("{:?}", panic_log);
    }));

    let cli = Cli::init();
    match cli.command {
        Commands::Init { config_path } => {
            tracing_subscriber::fmt().init();
            ConfigPath::init(&config_path)?;

            let database_path = config_path.as_ref().join(DATABASE_DIR_NAME);
            // Initialize the database
            let kv_store = KvStoreBuilder::default()
                .set_default_lock_timeout(5000)
                .set_txn_lock_timeout(5000)
                .build(database_path.clone())
                .map_err(error::Error::Database)?;
            kv_store.init();
            tracing::info!("Database initialized at {:?}", database_path);

            let mut version = Version::default();

            version.code_version = CURRENT_CODE_VERSION.to_string();
            version.database_version = REQURIED_DATABASE_VERSION.to_string();
            version.put().map_err(error::Error::Database)?;
        }
        Commands::Start { mut config_option } => {
            start_sequencer(&mut config_option).await?;
        }
    }

    Ok(())
}

async fn start_sequencer(config_option: &mut ConfigOption) -> Result<(), Error> {
    set_resource_limits()?;

    let config = Config::load(config_option)?;
    initialize_logger(&config)?;

    // Initialize the profiler.
    // let profiler = Profiler::init("http://127.0.0.1:4040", "sequencer", 100)?;
    let profiler = None;

    // Initialize the database
    let kv_store = KvStoreBuilder::default()
        .set_default_lock_timeout(5000)
        .set_txn_lock_timeout(5000)
        .build(config.database_path())
        .map_err(error::Error::Database)?;
    kv_store.init();
    tracing::info!("Database initialized at {:?}", config.database_path());

    let mut version = Version::get_mut_or(Version::default).map_err(error::Error::Database)?;

    if version.database_version != REQURIED_DATABASE_VERSION {
        tracing::error!(
            "Database version mismatch: expected {:?}, found {:?}",
            REQURIED_DATABASE_VERSION,
            version.database_version
        );
        return Err(error::Error::DatabaseVersionMismatch);
    }

    version.code_version = CURRENT_CODE_VERSION.to_string();
    tracing::info!("Current code version {:?}", version.code_version);
    tracing::info!("Current database version {:?}", version.database_version);
    version.update().map_err(error::Error::Database)?;

    let (seeder_client, distributed_key_generation_client) =
        tokio::try_join!(async { initialize_seeder_client(&config) }, async {
            initialize_dkg_client(&config)
        })?;
    let skde_params = distributed_key_generation_client
        .get_skde_params()
        .await?
        .skde_params;

    let rpc_client = RpcClient::new().map_err(error::Error::RpcClient)?;
    let merkle_tree_manager = MerkleTreeManager::init(&rpc_client).await;
    let app_state: AppState = AppState::new(
        config,
        seeder_client,
        distributed_key_generation_client,
        CachedKvStore::default(),
        CachedKvStore::default(),
        CachedKvStore::default(),
        skde_params,
        profiler,
        rpc_client,
        merkle_tree_manager,
    );

    initialize_clients(app_state.clone()).await?;

    let internal_handle = tokio::spawn(initialize_internal_rpc_server(app_state.clone()));
    let cluster_handle = tokio::spawn(initialize_cluster_rpc_server(app_state.clone()));
    let external_handle = tokio::spawn(initialize_external_rpc_server(app_state.clone()));

    let handles = vec![internal_handle, cluster_handle, external_handle];
    let results = try_join_all(handles).await;
    if let Err(e) = results {
        tracing::error!("One of the RPC servers terminated unexpectedly: {:?}", e);
        return Err(error::Error::RpcServerTerminated);
    }

    Ok(())
}

fn set_resource_limits() -> Result<(), Error> {
    let rlimit = get_resource_limit(ResourceType::RLIMIT_NOFILE)?;
    set_resource_limit(ResourceType::RLIMIT_NOFILE, rlimit.hard_limit)?;
    Ok(())
}

fn initialize_seeder_client(config: &Config) -> Result<SeederClient, Error> {
    let seeder_client = SeederClient::new(&config.seeder_rpc_url)?;
    tracing::info!("Seeder client initialized: {:?}", config.seeder_rpc_url);
    Ok(seeder_client)
}

fn initialize_dkg_client(config: &Config) -> Result<DistributedKeyGenerationClient, Error> {
    let dkg_client =
        DistributedKeyGenerationClient::new(&config.distributed_key_generation_rpc_url)?;
    tracing::info!(
        "Distributed Key Generation client initialized: {:?}",
        config.distributed_key_generation_rpc_url
    );
    Ok(dkg_client)
}

async fn initialize_clients(app_state: AppState) -> Result<(), Error> {
    let sequencing_info_list =
        SequencingInfoList::get_or(SequencingInfoList::default).map_err(Error::Database)?;

    for (platform, service_provider) in sequencing_info_list.iter() {
        let sequencing_info_payload =
            SequencingInfoPayload::get(*platform, *service_provider).map_err(Error::Database)?;

        match sequencing_info_payload {
            SequencingInfoPayload::Ethereum(liveness_info) => {
                liveness_service_manager::radius::LivenessServiceManagerClient::initialize(
                    app_state.clone(),
                    *platform,
                    *service_provider,
                    liveness_info,
                )
                .await?;
            }
            SequencingInfoPayload::Local(_payload) => {
                tracing::warn!(
                    "Local LivenessServiceManagerClient not implemented for platform {:?} and service provider {:?}",
                    platform,
                    service_provider
                );
                todo!("Implement 'LivenessServiceManagerClient' for local sequencing.");
            }
        }
    }

    let validation_service_providers =
        ValidationServiceProviders::get_or(ValidationServiceProviders::default)
            .map_err(Error::Database)?;

    for (platform, provider) in validation_service_providers.iter() {
        let validation_info = ValidationInfo::get(*platform, *provider).map_err(Error::Database)?;
        match validation_info {
            ValidationInfo::EigenLayer(info) => {
                validation_service_manager::eigenlayer::ValidationServiceManagerClient::initialize(
                    app_state.clone(),
                    *platform,
                    *provider,
                    info,
                );
            }
            ValidationInfo::Symbiotic(info) => {
                validation_service_manager::symbiotic::ValidationServiceManagerClient::initialize(
                    app_state.clone(),
                    *platform,
                    *provider,
                    info,
                );
            }
        }
    }

    Ok(())
}

async fn initialize_internal_rpc_server(context: AppState) -> Result<(), Error> {
    let internal_rpc_url = context.config().internal_rpc_url.to_string();

    let internal_rpc_server = RpcServer::new(context.clone())
        .register_rpc_method::<internal::AddSequencingInfo>()?
        .register_rpc_method::<internal::AddValidationInfo>()?
        .register_rpc_method::<internal::AddCluster>()?
        .register_rpc_method::<internal::GetCluster>()?
        .register_rpc_method::<internal::GetClusterIdList>()?
        .register_rpc_method::<internal::GetSequencingInfos>()?
        .register_rpc_method::<internal::GetSequencingInfo>()?
        .register_rpc_method::<internal::SetMaxGasLimit>()?
        .init(internal_rpc_url.clone())
        .await?;

    tracing::info!(
        "Successfully started the internal RPC server: {}",
        internal_rpc_url
    );

    internal_rpc_server.stopped().await;
    Ok(())
}

async fn initialize_cluster_rpc_server(context: AppState) -> Result<(), Error> {
    let cluster_rpc_url = anywhere(&context.config().cluster_port()?);

    let cluster_rpc_server = RpcServer::new(context)
        .register_rpc_method::<cluster::SyncEncryptedTransaction>()?
        .register_rpc_method::<cluster::SyncRawTransaction>()?
        .register_rpc_method::<cluster::FinalizeBlock>()?
        .register_rpc_method::<cluster::SyncBlock>()?
        .register_rpc_method::<cluster::SyncMaxGasLimit>()?
        .register_rpc_method::<external::GetRawTransactionList>()?
        .init(cluster_rpc_url.clone())
        .await?;

    tracing::info!(
        "Successfully started the cluster RPC server: {}",
        cluster_rpc_url
    );

    cluster_rpc_server.stopped().await;
    Ok(())
}

async fn initialize_external_rpc_server(context: AppState) -> Result<(), Error> {
    let external_rpc_url = anywhere(&context.config().external_port()?);

    tracing::info!(
        "Successfully started the sequencer external RPC server: {}",
        external_rpc_url
    );

    let external_rpc_server = RpcServer::new(context)
        .register_rpc_method::<external::SendEncryptedTransaction>()?
        .register_rpc_method::<external::GetEncryptedTransactionWithTransactionHash>()?
        .register_rpc_method::<external::GetEncryptedTransactionWithOrderCommitment>()?
        .register_rpc_method::<external::GetRawTransactionWithTransactionHash>()?
        .register_rpc_method::<external::GetRawTransactionWithOrderCommitment>()?
        .register_rpc_method::<external::GetOrderCommitment>()?
        .register_rpc_method::<external::SendRawTransaction>()?
        .register_rpc_method::<external::GetRawTransactionList>()?
        .register_rpc_method::<external::GetEncryptedTransactionList>()?
        .register_rpc_method::<external::GetRollup>()?
        .register_rpc_method::<external::GetRollupMetadata>()?
        .register_rpc_method::<external::GetBlock>()?
        .register_rpc_method::<external::GetBlockHeight>()?
        .register_rpc_method::<external::GetVersion>()?
        .init(external_rpc_url)
        .await?;

    external_rpc_server.stopped().await;
    Ok(())
}

pub fn anywhere(port: &str) -> String {
    format!("0.0.0.0:{}", port)
}
