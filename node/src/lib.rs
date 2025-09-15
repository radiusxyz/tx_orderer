pub mod types;
pub mod utils;
pub mod rpc;
pub mod tasks;
pub mod clients;
pub mod state;
pub mod services;

pub use types::*;
pub use utils::*;
pub use rpc::{
    AddMevSearcherInfo, AddMevSearcherInfoMessage, BatchCreationMessage,
    GetBatch, GetBatchResponse, GetCanProvideTransactionInfo,
    GetCanProvideTransactionInfoResponse, GetClusterMetadata,
    GetClusterMetadataResponse, GetEncryptedTransactionList,
    GetEncryptedTransactionListResponse, GetEncryptedTransactionWithOrderCommitment,
    GetEncryptedTransactionWithTransactionHash, GetOrderCommitment,
    GetOrderCommitmentInfo, GetOrderCommitmentInfoResponse,
    GetOrderCommitmentResponse, GetPostMerklePath,
    ClusterGetRawTransactionList, ClusterGetRawTransactionListResponse,
    ExternalGetRawTransactionList, ExternalGetRawTransactionListResponse,
    GetRawTransactionWithOrderCommitment, GetRawTransactionWithOrderCommitmentResponse,
    GetRawTransactionWithTransactionHash, GetRawTransactionWithTransactionHashResponse,
    GetRollup, GetRollupMetadata, GetRollupMetadataResponse, GetRollupResponse,
    GetVersion, GetVersionResponse, LeaderChangeMessage,
    RemoveMevSearcherInfo, RemoveMevSearcherInfoMessage,
    SendEncryptedTransaction, SendRawTransaction, SetLeaderTxOrderer,
    SetMaxGasLimit, SignMessage, SyncBatchCreation,
    SyncEncryptedTransaction, SyncLeaderTxOrderer, SyncMaxGasLimit,
    SyncMaxGasLimitMessage, SyncRawTransaction,
    sync_batch_creation, sync_encrypted_transaction, sync_raw_transaction,
};
pub use tasks::*;
pub use clients::*;
pub use state::*;
pub use services::*;

// Public API functions for CLI
use std::{collections::HashMap, sync::{Arc, Mutex}};
use futures::future::try_join_all;
use radius_sdk::{
    json_rpc::{client::RpcClient, server::RpcServer},
    kvstore::{CachedKvStore, KvStoreBuilder},
    util::{get_resource_limit, set_resource_limit, ResourceType},
};
use tx_orderer_primitives::{error::{self, Error}, ValidationInfo, Version, SequencingInfoPayload, ValidationServiceProviders, SequencingInfoList, REQURIED_DATABASE_VERSION, CURRENT_CODE_VERSION};

// Error conversions for this crate
impl From<ConfigError> for Error {
    fn from(value: ConfigError) -> Self {
        Self::Config(format!("{:?}", value))
    }
}

impl From<SeederError> for Error {
    fn from(value: SeederError) -> Self {
        Self::Seeder(format!("{:?}", value))
    }
}

impl From<DistributedKeyGenerationClientError> for Error {
    fn from(value: DistributedKeyGenerationClientError) -> Self {
        Self::DistributedKeyGeneration(format!("{:?}", value))
    }
}

impl From<RewardManagerError> for Error {
    fn from(value: RewardManagerError) -> Self {
        Self::RewardManager(format!("{:?}", value))
    }
}

pub async fn init_node(config_path: &ConfigPath) -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    ConfigPath::init(&config_path)?;

    let database_path = config_path.as_ref().join(DATABASE_DIR_NAME);
    let kv_store = KvStoreBuilder::default()
        .set_default_lock_timeout(10000)
        .set_txn_lock_timeout(10000)
        .build(database_path.clone())
        .map_err(error::Error::Database)?;
    kv_store.init();
    tracing::info!("Database initialized at {:?}", database_path);

    let mut version = Version::default();
    version.code_version = CURRENT_CODE_VERSION.to_string();
    version.database_version = REQURIED_DATABASE_VERSION.to_string();
    version.put().map_err(error::Error::Database)?;
    
    Ok(())
}

pub async fn start_node(config_option: &mut ConfigOption) -> Result<(), Error> {
    set_resource_limits()?;
    let config = Config::load(config_option)?;
    tracing_subscriber::fmt().init();

    // Initialize the profiler.
    let profiler = None;

    let kv_store = KvStoreBuilder::default()
        .set_default_lock_timeout(10000)
        .set_txn_lock_timeout(10000)
        .build(config.database_path())
        .map_err(error::Error::Database)?;
    kv_store.init();
    tracing::info!("Database initialized at {:?}", config.database_path());

    check_and_update_version()?;

    let (seeder_client, dkg_client, reward_manager_client) = tokio::try_join!(
        async { initialize_seeder_client(&config) },
        async { initialize_dkg_client(&config) },
        async { initialize_reward_manager_client(&config) }
    )?;
    let skde_params = dkg_client.get_skde_params().await?.skde_params;
    let latest_key_id = dkg_client.get_latest_key_id().await?.latest_key_id;

    let decryptor = tasks::Decryptor::new(
        dkg_client.clone(),
        skde_params.clone(),
        latest_key_id,
        config.builder_rpc_url.clone(),
    )?;
    tasks::Decryptor::start(decryptor.clone()).await;

    let rpc_client = RpcClient::new().map_err(error::Error::RpcClient)?;
    let merkle_tree_manager = utils::MerkleTreeManager::init(&rpc_client).await;
    let app_state = AppState::new(
        config,
        seeder_client,
        reward_manager_client,
        decryptor,
        CachedKvStore::default(),
        CachedKvStore::default(),
        CachedKvStore::default(),
        skde_params,
        profiler,
        rpc_client,
        merkle_tree_manager,
        Arc::new(Mutex::new(HashMap::new())),
    );

    tasks::run_backrunning_server(app_state.shared_channel_infos().clone()).await;

    // Start all background services
    services::ServiceManager::start_all_services(app_state.clone()).await;

    initialize_node_clients(app_state.clone()).await?;

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

fn initialize_seeder_client(config: &Config) -> Result<clients::SeederClient, Error> {
    let seeder_client = clients::SeederClient::new(&config.seeder_rpc_url)?;
    tracing::info!("Seeder client initialized: {:?}", config.seeder_rpc_url);
    Ok(seeder_client)
}

fn initialize_dkg_client(config: &Config) -> Result<clients::DistributedKeyGenerationClient, Error> {
    let dkg_client = clients::DistributedKeyGenerationClient::new(&config.distributed_key_generation_rpc_url)?;
    tracing::info!(
        "Distributed Key Generation client initialized: {:?}",
        config.distributed_key_generation_rpc_url
    );
    Ok(dkg_client)
}

fn initialize_reward_manager_client(config: &Config) -> Result<clients::RewardManagerClient, Error> {
    let reward_manager_client = clients::RewardManagerClient::new(&config.reward_manager_rpc_url)?;
    tracing::info!(
        "Reward Manager client initialized: {:?}",
        config.reward_manager_rpc_url
    );
    Ok(reward_manager_client)
}

async fn initialize_node_clients(app_state: AppState) -> Result<(), Error> {
    let sequencing_info_list =
        SequencingInfoList::get_or(SequencingInfoList::default).map_err(Error::Database)?;

    for (platform, service_provider) in sequencing_info_list.iter() {
        let sequencing_info_payload =
            SequencingInfoPayload::get(*platform, *service_provider).map_err(Error::Database)?;

        match sequencing_info_payload {
            SequencingInfoPayload::Ethereum(liveness_info) => {
                clients::liveness_service_manager::radius::LivenessServiceManagerClient::initialize(
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

    for (platform, validation_service_provider) in validation_service_providers.iter() {
        let validation_info = ValidationInfo::get(*platform, *validation_service_provider)
            .map_err(Error::Database)?;
        match validation_info {
            ValidationInfo::EigenLayer(info) => {
                clients::validation_service_manager::eigenlayer::ValidationServiceManagerClient::initialize(
                    app_state.clone(),
                    *platform,
                    *validation_service_provider,
                    info,
                );
            }
            ValidationInfo::Symbiotic(info) => {
                clients::validation_service_manager::symbiotic::ValidationServiceManagerClient::initialize(
                    app_state.clone(),
                    *platform,
                    *validation_service_provider,
                    info,
                );
            }
        }
    }

    Ok(())
}

async fn initialize_internal_rpc_server(context: AppState) -> Result<(), Error> {
    let internal_rpc_url = context.config().internal_rpc_url.to_string();
    let internal_rpc_server = Arc::new(RpcServer::new(context.clone()));

    internal_rpc_server.register_rpc_method::<rpc::internal::AddSequencingInfo>().await?;
    internal_rpc_server.register_rpc_method::<rpc::internal::AddValidationInfo>().await?;
    internal_rpc_server.register_rpc_method::<rpc::internal::AddCluster>().await?;
    internal_rpc_server.register_rpc_method::<rpc::internal::GetCluster>().await?;
    internal_rpc_server.register_rpc_method::<rpc::internal::GetClusterIdList>().await?;
    internal_rpc_server.register_rpc_method::<rpc::internal::GetSequencingInfos>().await?;
    internal_rpc_server.register_rpc_method::<rpc::internal::GetSequencingInfo>().await?;

    let internal_handle = internal_rpc_server.init(internal_rpc_url.clone()).await?;
    tracing::info!("Successfully started the internal RPC server: {}", internal_rpc_url);
    internal_handle.stopped().await;
    Ok(())
}

async fn initialize_cluster_rpc_server(context: AppState) -> Result<(), Error> {
    let cluster_rpc_url = anywhere(&context.config().cluster_port()?);
    let cluster_rpc_server = Arc::new(RpcServer::new(context.clone()));

    cluster_rpc_server.register_rpc_method::<rpc::cluster::GetRawTransactionList>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::SetMaxGasLimit>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::SyncEncryptedTransaction>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::GetOrderCommitmentInfo>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::SyncLeaderTxOrderer>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::SyncRawTransaction>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::SyncMaxGasLimit>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::SyncBatchCreation>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::AddMevSearcherInfo>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::RemoveMevSearcherInfo>().await?;
    cluster_rpc_server.register_rpc_method::<rpc::cluster::SetLeaderTxOrderer>().await?;

    let cluster_handle = cluster_rpc_server.init(cluster_rpc_url.clone()).await?;
    tracing::info!("Successfully started the cluster RPC server: {}", cluster_rpc_url);
    cluster_handle.stopped().await;
    Ok(())
}

async fn initialize_external_rpc_server(context: AppState) -> Result<(), Error> {
    let external_rpc_url = anywhere(&context.config().external_port()?);
    tracing::info!("Successfully started the tx_orderer external RPC server: {}", external_rpc_url);
    let external_rpc_server = Arc::new(RpcServer::new(context.clone()));

    external_rpc_server.register_rpc_method::<rpc::external::SendEncryptedTransaction>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetEncryptedTransactionWithTransactionHash>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetEncryptedTransactionWithOrderCommitment>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetRawTransactionWithTransactionHash>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetRawTransactionWithOrderCommitment>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetOrderCommitment>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::SendRawTransaction>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetRawTransactionList>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetEncryptedTransactionList>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetRollup>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetRollupMetadata>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetClusterMetadata>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetVersion>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetBatch>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetPostMerklePath>().await?;
    external_rpc_server.register_rpc_method::<rpc::external::GetCanProvideTransactionInfo>().await?;

    let external_handle = external_rpc_server.init(external_rpc_url.clone()).await?;
    external_handle.stopped().await;
    Ok(())
}

pub fn anywhere(port: &str) -> String {
    format!("0.0.0.0:{}", port)
}

fn check_and_update_version() -> Result<(), Error> {
    let mut version = Version::get_mut_or(Version::default).map_err(error::Error::Database)?;
    if version.database_version != REQURIED_DATABASE_VERSION {
        tracing::error!(
            "Database version mismatch: expected {}, found {}",
            REQURIED_DATABASE_VERSION,
            version.database_version
        );
        return Err(error::Error::DatabaseVersionMismatch);
    }
    version.code_version = CURRENT_CODE_VERSION.to_string();
    version.update().map_err(error::Error::Database)?;
    Ok(())
}