use clap::{Parser, Subcommand};
use radius_sdk::{
    json_rpc::{client::RpcClient, server::RpcServer},
    kvstore::{CachedKvStore, KvStoreBuilder},
    util::{get_resource_limit, set_resource_limit, ResourceType},
};
use sequencer::{
    client::{
        liveness::{
            self, distributed_key_generation::DistributedKeyGenerationClient, seeder::SeederClient,
        },
        validation,
    },
    error::{self, Error},
    logger::{Logger, PanicLog},
    merkle_tree_manager::MerkleTreeManager,
    rpc::{cluster, external, internal},
    state::AppState,
    types::*,
};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[derive(Debug, Deserialize, Parser, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn init() -> Self {
        Cli::parse()
    }
}

#[derive(Subcommand, Debug, Deserialize, Serialize)]
pub enum Commands {
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
            ConfigPath::init(&config_path)?
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
    KvStoreBuilder::default()
        .set_default_lock_timeout(5000)
        .set_txn_lock_timeout(5000)
        .build(config.database_path())
        .map_err(error::Error::Database)?
        .init();

    tracing::info!("Database initialized at {:?}", config.database_path());

    let seeder_client = initialize_seeder_client(&config)?;
    let distributed_key_generation_client = initialize_dkg_client(&config)?;

    let skde_params = distributed_key_generation_client
        .get_skde_params()
        .await?
        .skde_params;

    let merkle_tree_manager = MerkleTreeManager::init().await;
    let app_state: AppState = AppState::new(
        config,
        seeder_client,
        distributed_key_generation_client,
        CachedKvStore::default(),
        CachedKvStore::default(),
        CachedKvStore::default(),
        skde_params,
        profiler,
        RpcClient::new().unwrap(),
        merkle_tree_manager,
    );

    initialize_clients(app_state.clone()).await?;
    initialize_rpc_servers(app_state).await?;

    Ok(())
}

fn set_resource_limits() -> Result<(), Error> {
    let rlimit = get_resource_limit(ResourceType::RLIMIT_NOFILE)?;
    set_resource_limit(ResourceType::RLIMIT_NOFILE, rlimit.hard_limit)?;
    Ok(())
}

fn initialize_logger(config: &Config) -> Result<(), Error> {
    Logger::new(config.log_path())
        .map_err(error::Error::Logger)?
        .init();
    tracing::info!("Logger initialized.");
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
                liveness::radius::LivenessClient::initialize(
                    app_state.clone(),
                    *platform,
                    *service_provider,
                    liveness_info,
                );
            }
            SequencingInfoPayload::Local(_payload) => {
                todo!("Implement 'LivenessClient' for local sequencing.");
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
                validation::eigenlayer::ValidationClient::initialize(
                    app_state.clone(),
                    *platform,
                    *provider,
                    info,
                );
            }
            ValidationInfo::Symbiotic(info) => {
                validation::symbiotic::ValidationClient::initialize(
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

async fn initialize_rpc_servers(app_state: AppState) -> Result<(), Error> {
    initialize_internal_rpc_server(app_state.clone()).await?;
    initialize_cluster_rpc_server(app_state.clone()).await?;
    initialize_external_rpc_server(app_state)
        .await?
        .await
        .unwrap();
    Ok(())
}

async fn initialize_internal_rpc_server(context: AppState) -> Result<(), Error> {
    let internal_rpc_url = context.config().internal_rpc_url.to_string();

    // Initialize the internal RPC server.
    let internal_rpc_server = RpcServer::new(context.clone())
        .register_rpc_method::<internal::AddSequencingInfo>()?
        .register_rpc_method::<internal::AddValidationInfo>()?
        .register_rpc_method::<internal::AddCluster>()?
        .register_rpc_method::<internal::GetCluster>()?
        .register_rpc_method::<internal::GetClusterIdList>()?
        .register_rpc_method::<internal::GetSequencingInfos>()?
        .register_rpc_method::<internal::GetSequencingInfo>()?
        .init(internal_rpc_url.clone())
        .await?;

    tracing::info!(
        "Successfully started the internal RPC server: {}",
        internal_rpc_url
    );

    tokio::spawn(internal_rpc_server.stopped());

    Ok(())
}

async fn initialize_cluster_rpc_server(context: AppState) -> Result<(), Error> {
    let cluster_rpc_url = anywhere(&context.config().cluster_port()?);

    let cluster_rpc_server = RpcServer::new(context)
        .register_rpc_method::<cluster::SyncEncryptedTransaction>()?
        .register_rpc_method::<cluster::SyncRawTransaction>()?
        .register_rpc_method::<cluster::FinalizeBlock>()?
        .register_rpc_method::<cluster::SyncBlock>()?
        .register_rpc_method::<external::GetRawTransactionList>()?
        .init(cluster_rpc_url.clone())
        .await?;

    tracing::info!(
        "Successfully started the cluster RPC server: {}",
        cluster_rpc_url
    );

    tokio::spawn(cluster_rpc_server.stopped());

    Ok(())
}

async fn initialize_external_rpc_server(context: AppState) -> Result<JoinHandle<()>, Error> {
    let external_rpc_url = anywhere(&context.config().external_port()?);

    tracing::info!(
        "Successfully started the sequencer external RPC server: {}",
        external_rpc_url
    );

    // Initialize the external RPC server.
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
        .init(external_rpc_url)
        .await?;

    let server_handle = tokio::spawn(async move {
        external_rpc_server.stopped().await;
    });

    Ok(server_handle)
}

pub fn anywhere(port: &str) -> String {
    format!("0.0.0.0:{}", port)
}
