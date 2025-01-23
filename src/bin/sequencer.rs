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
    // profiler::Profiler,
    rpc::{cluster, external, internal},
    state::AppState,
    types::*,
};
pub use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[derive(Debug, Deserialize, Parser, Serialize)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
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
        config_path: Box<ConfigPath>,
    },

    /// Starts the node
    Start {
        #[clap(flatten)]
        config_option: Box<ConfigOption>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::panic::set_hook(Box::new(|panic_info| {
        let panic_log: PanicLog = panic_info.into();
        tracing::error!("{:?}", panic_log);
    }));

    let mut cli = Cli::init();
    match cli.command {
        Commands::Init { ref config_path } => {
            tracing_subscriber::fmt().init();
            ConfigPath::init(config_path)?
        }
        Commands::Start {
            ref mut config_option,
        } => {
            start_sequencer(config_option).await?;
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

    let app_state: AppState = AppState::new(
        config,
        seeder_client,
        distributed_key_generation_client,
        CachedKvStore::default(),
        CachedKvStore::default(),
        CachedKvStore::default(),
        skde_params,
        profiler,
        CachedKvStore::default(),
        CachedKvStore::default(),
        CachedKvStore::default(),
        RpcClient::new().unwrap(),
    );

    initialize_rollups(app_state.clone()).await?;
    initialize_rollup_metadatas(app_state.clone()).await?;
    initialize_clients(app_state.clone()).await?;
    initialize_rpc_servers(app_state).await?;

    Ok(())
}

fn set_resource_limits() -> Result<(), Error> {
    let rlimit = get_resource_limit(ResourceType::RLIMIT_NOFILE)?;
    set_resource_limit(ResourceType::RLIMIT_NOFILE, rlimit.hard_limit)?;
    Ok(())
}

#[allow(dead_code)]
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

async fn initialize_rollup_metadatas(app_state: AppState) -> Result<(), Error> {
    let rollup_id_list = RollupIdList::get_or(RollupIdList::default).map_err(Error::Database)?;

    for rollup_id in rollup_id_list.iter() {
        let rollup_metadata = RollupMetadata::get(rollup_id).map_err(Error::Database)?;

        app_state
            .add_rollup_metadata(&rollup_id, rollup_metadata)
            .await?;
    }

    Ok(())
}

async fn initialize_rollups(app_state: AppState) -> Result<(), Error> {
    let rollup_id_list = RollupIdList::get_or(RollupIdList::default).map_err(Error::Database)?;

    for rollup_id in rollup_id_list.iter() {
        let rollup = Rollup::get(rollup_id).map_err(Error::Database)?;

        app_state.add_rollup(&rollup_id, rollup).await?;
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

    tokio::spawn(async move {
        internal_rpc_server.stopped().await;
    });

    Ok(())
}

async fn initialize_cluster_rpc_server(context: AppState) -> Result<(), Error> {
    let cluster_rpc_url = anywhere(&context.config().cluster_port()?);

    tracing::info!(
        "Successfully started the cluster RPC server: {}",
        cluster_rpc_url
    );

    let sequencer_rpc_server = RpcServer::new(context)
        .register_rpc_method::<cluster::SyncEncryptedTransaction>()?
        .register_rpc_method::<cluster::SyncRawTransaction>()?
        .register_rpc_method::<cluster::FinalizeBlock>()?
        .register_rpc_method::<cluster::SyncBlock>()?
        .register_rpc_method::<external::GetRawTransactionList>()?
        .init(cluster_rpc_url)
        .await?;

    tokio::spawn(async move {
        sequencer_rpc_server.stopped().await;
    });

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

// TODO
// #[cfg(test)]
// mod tests {
//     use std::{
//         collections::BTreeMap,
//         sync::{Arc, Mutex},
//         thread,
//     };

//     use super::*;

//     #[test]
//     fn test_multithreaded_execution_mutex() {
//         KvStoreBuilder::default()
//             .set_default_lock_timeout(5000)
//             .set_txn_lock_timeout(5000)
//             .build("./test_db")
//             .map_err(error::Error::Database)
//             .unwrap()
//             .init();

//         // 공유 자원 (Arc + Mutex를 사용하여 스레드 간 안전하게 공유)
//         let shared_cluster_id_list = Arc::new(Mutex::new(BTreeMap::<
//             (Platform, ServiceProvider),
//             ClusterIdList,
//         >::new()));

//         // 스레드 풀 크기
//         let thread_count = 4;

//         // 스레드 핸들 저장
//         let mut handles = vec![];

//         let start = Instant::now();
//         for i in 0..thread_count {
//             let cluster_id_list_clone: Arc<
//                 Mutex<BTreeMap<(Platform, ServiceProvider), ClusterIdList>>,
//             > = Arc::clone(&shared_cluster_id_list);

//             // 스레드 생성
//             let handle = thread::spawn(move || {
//                 for j in 0..1000 {
//                     let mut cluster_id_list =
// cluster_id_list_clone.lock().unwrap();                     let cluster =
//                         cluster_id_list.get_mut(&(Platform::Ethereum,
// ServiceProvider::Radius));

//                     let cluster_id = format!("cluster_id_{}_{}", i, j);
//                     match cluster {
//                         Some(cluster_id_list) => {
//                             cluster_id_list.insert(&cluster_id);
//                         }
//                         None => {
//                             cluster_id_list.insert(
//                                 (Platform::Ethereum,
// ServiceProvider::Radius),
// ClusterIdList::default(),                             );
//                         }
//                     }
//                 }

//                 // for j in 0..1000 {
//                 //     let cluster_id_list =
//                 // cluster_id_list_clone.lock().unwrap(); }
//             });

//             handles.push(handle);
//         }

//         // 모든 스레드가 완료될 때까지 대기
//         for handle in handles {
//             handle.join().expect("Thread panicked");
//         }

//         let duration = start.elapsed();
//         println!("{:?}", shared_cluster_id_list.lock().unwrap());
//         println!("Time elapsed: {:?}", duration);
//     }

//     #[test]
//     fn test_multithreaded_execution_db() {
//         KvStoreBuilder::default()
//             .set_default_lock_timeout(5000)
//             .set_txn_lock_timeout(5000)
//             .build("./test_db")
//             .map_err(error::Error::Database)
//             .unwrap()
//             .init();

//         // 스레드 풀 크기
//         let thread_count = 4;

//         // 스레드 핸들 저장
//         let mut handles = vec![];

//         let start = Instant::now();
//         for i in 0..thread_count {
//             let handle = thread::spawn(move || {
//                 // for j in 0..1000 {
//                 //     let cluster_id = format!("cluster_id_{}_{}", i, j);
//                 //     let mut cluster_id_list = ClusterIdList::get_mut_or(
//                 //         Platform::Ethereum,
//                 //         ServiceProvider::Radius,
//                 //         ClusterIdList::default,
//                 //     )
//                 //     .unwrap();
//                 //     cluster_id_list.insert(&cluster_id);
//                 //     cluster_id_list.update().unwrap();
//                 // }

//                 // for j in 0..1000 {
//                 //     let cluster_id_list = ClusterIdList::get_or(
//                 //         Platform::Ethereum,
//                 //         ServiceProvider::Radius,
//                 //         ClusterIdList::default,
//                 //     )
//                 //     .unwrap();
//                 // }

//                 for j in 0..1000 {
//                     let cluster_id_list = ClusterIdList::get_mut_or(
//                         Platform::Ethereum,
//                         ServiceProvider::Radius,
//                         ClusterIdList::default,
//                     )
//                     .unwrap();
//                 }
//             });

//             handles.push(handle);
//         }

//         for handle in handles {
//             handle.join().expect("Thread panicked");
//         }

//         let duration = start.elapsed();
//         println!("Time elapsed: {:?}", duration);

//         let cluster_id_list = ClusterIdList::get_or(
//             Platform::Ethereum,
//             ServiceProvider::Radius,
//             ClusterIdList::default,
//         )
//         .unwrap();

//         println!("{:?}", cluster_id_list);
//     }

//     #[tokio::test]
//     async fn test_multithreaded_execution_memory_db() {
//         // 스레드 풀 크기
//         let thread_count = 4;

//         // 스레드 핸들 저장
//         let mut handles = vec![];

//         //

//         let memory_db = CachedKvStore::default();

//         let _ = memory_db
//             .put(
//                 &(Platform::Ethereum, ServiceProvider::Radius),
//                 ClusterIdList::default(),
//             )
//             .await;

//         let shared_cluster_id_list = Arc::new(memory_db);

//         let start = Instant::now();
//         for i in 0..thread_count {
//             let cluster_id_list_clone = Arc::clone(&shared_cluster_id_list);

//             let handle = thread::spawn(move || async move {
//                 for j in 0..1000 {
//                     let cluster_id = format!("cluster_id_{}_{}", i, j);

//                     let mut cluster_id_list: Value<ClusterIdList> =
// cluster_id_list_clone                         .as_ref()
//                         .get_mut(&(Platform::Ethereum,
// ServiceProvider::Radius))                         .await
//                         .unwrap();

//                     cluster_id_list.insert(&cluster_id);
//                     drop(cluster_id_list);
//                 }

//                 // for j in 0..1000 {
//                 //     let cluster_id_list = ClusterIdList::get_or(
//                 //         Platform::Ethereum,
//                 //         ServiceProvider::Radius,
//                 //         ClusterIdList::default,
//                 //     )
//                 //     .unwrap();
//                 // }

//                 // for j in 0..1000 {
//                 //     let cluster_id_list = ClusterIdList::get_mut_or(
//                 //         Platform::Ethereum,
//                 //         ServiceProvider::Radius,
//                 //         ClusterIdList::default,
//                 //     )
//                 //     .unwrap();
//                 // }
//             });

//             handles.push(handle);
//         }

//         for handle in handles {
//             handle.join().expect("Thread panicked").await;
//         }

//         let duration = start.elapsed();

//         let cluster_id_list: Value<ClusterIdList> = shared_cluster_id_list
//             .as_ref()
//             .get_mut(&(Platform::Ethereum, ServiceProvider::Radius))
//             .await
//             .unwrap();
//         println!("{:?}", cluster_id_list);
//         println!("Time elapsed: {:?}", duration);
//     }
// }

// #[cfg(test)]
// mod tests2 {
//     use std::{
//         collections::{BTreeMap, BTreeSet},
//         sync::Mutex,
//         thread,
//     };

//     use external::issue_order_commitment;
//     use liveness::seeder::SequencerRpcInfo;
//     use radius_sdk::{
//         json_rpc::client::RpcClient,
//         kvstore::{CachedKvStore, KvStoreBuilder},
//         signature::{Address, ChainType, PrivateKeySigner},
//     };

//     use super::*;

//     #[tokio::test]
//     async fn test_send_raw_transaction() {
//         // tracing_subscriber::fmt().init();
//         // std::panic::set_hook(Box::new(|panic_info| {
//         //     let panic_log: PanicLog = panic_info.into();
//         //     tracing::error!("{:?}", panic_log);
//         // }));
//         // 15 -> 6

//         let config = Config::default();
//         KvStoreBuilder::default()
//             .set_default_lock_timeout(5000)
//             .set_txn_lock_timeout(5000)
//             .build("./test_db")
//             .unwrap()
//             .init();

//         let seeder_client = initialize_seeder_client(&config).unwrap();
//         let distributed_key_generation_client =
// initialize_dkg_client(&config).unwrap();

//         let skde_params = distributed_key_generation_client
//             .get_skde_params()
//             .await
//             .unwrap()
//             .skde_params;

//         let context: AppState = AppState::new(
//             config,
//             seeder_client,
//             distributed_key_generation_client,
//             CachedKvStore::default(),
//             CachedKvStore::default(),
//             CachedKvStore::default(),
//             skde_params,
//             None,
//             // Mutex::new(BTreeMap::new()),
//             CachedKvStore::default(),
//             CachedKvStore::default(),
//             CachedKvStore::default(),
//             RpcClient::new().unwrap(),
//         );

//         ////////////////////////////////////////////////

//         let address = Address::from_str(
//             ChainType::Ethereum,
//             "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
//         )
//         .unwrap();

//         let cluster_id = "cluster_id".to_string();
//         let platform = Platform::Ethereum;
//         let service_provider = ServiceProvider::Radius;
//         let platform_block_height = 7;

//         let signing_key = &context.config().signing_key;
//         let signer = PrivateKeySigner::from_str(platform.into(),
// signing_key).unwrap();

//         context.add_signer(platform, signer).await.unwrap();

//         let rollup_id = "rollup_id".to_string();
//         let mut rollup_id_list = BTreeSet::new();
//         rollup_id_list.insert(rollup_id.clone());

//         let mut sequencer_rpc_infos = BTreeMap::new();
//         let mut sequencer_rpc_info = SequencerRpcInfo::default();
//         sequencer_rpc_info.address = address.clone();

//         sequencer_rpc_infos.insert(1, sequencer_rpc_info);

//         let rollup = Rollup::new(
//             rollup_id.clone(),
//             RollupType::PolygonCdk,
//             EncryptedTransactionType::Skde,
//             address.clone(),
//             RollupValidationInfo::new(
//                 Platform::Ethereum,
//                 ValidationServiceProvider::Symbiotic,
//                 address.clone(),
//             ),
//             OrderCommitmentType::Sign,
//             vec![address.clone()],
//             "cluster_id".to_string(),
//             Platform::Ethereum,
//             ServiceProvider::Radius,
//         );
//         rollup.put(&rollup_id).unwrap();

//         let cluster = Cluster::new(
//             sequencer_rpc_infos,
//             rollup_id_list,
//             Address::from_str(
//                 ChainType::Ethereum,
//                 "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
//             )
//             .unwrap(),
//             7,
//         );
//         let _ = cluster.put(
//             platform,
//             service_provider,
//             &cluster_id,
//             platform_block_height,
//         );

//         let mut rollup_metadata = RollupMetadata::default();

//         rollup_metadata.cluster_id = rollup.cluster_id;
//         rollup_metadata.rollup_block_height = 1;
//         rollup_metadata.platform_block_height = platform_block_height;
//         rollup_metadata.is_leader = true;
//         rollup_metadata.leader_sequencer_rpc_info =
//             cluster.get_sequencer_rpc_info(&address).unwrap();
//         rollup_metadata.new_merkle_tree();
//         rollup_metadata.put(&rollup_id).unwrap();

//         context
//             .add_rollup_metadata(&rollup_id, rollup_metadata)
//             .await
//             .unwrap();

//         // let transaction = TransactionRequest {
//         //     chain_id: Some(1001),
//         //     to: Some(TxKind::Call(to.address())),
//         //     nonce: Some(from.fetch_add_nonce()),
//         //     gas: Some(21_000),
//         //     gas_price: Some(1_000_000_000),
//         //     value: Some(U256::from(1)),
//         //     ..Default::default()
//         // };

//         let raw_transaction =
// RawTransaction::Eth(EthRawTransaction("
// 0xf8ac832d4e1f8467e30ea28310059094dac17f958d2ee523a2206206994597c13d831ec780b844a9059cbb0000000000000000000000004ed07c2a3d0dec96f93663d6b71cb34294830ad50000000000000000000000000000000000000000000000000000000002fa8edb26a021f06184dd61bd7b87f8d4fc56bc209dbb1a4e0ce3d5cb1e6a56c55f9cf60620a01e1ae25306ebb1419a41c67db3e511d610fc9a7d83f332b7d7e3c8df951c4a5d"
// .to_string()));

//         ////////////////////////////////////////////////

//         let thread_count = 200;

//         let mut handles = Vec::new();

//         let start = Instant::now();
//         for i in 0..thread_count {
//             let context_clone = context.clone();
//             let rollup_id = rollup_id.clone();
//             let raw_transaction = raw_transaction.clone();

//             let handle = tokio::spawn(async move {
//                 for j in 0..25 {
//                     send_raw_transaction(
//                         context_clone.clone(),
//                         rollup_id.clone(),
//                         raw_transaction.clone(),
//                         j,
//                     )
//                     .await
//                     .unwrap();
//                 }
//             });

//             handles.push(handle);
//         }

//         // 모든 작업 대기
//         for handle in handles {
//             if let Err(err) = handle.await {
//                 eprintln!("Task failed: {:?}", err);
//             }
//         }

//         let duration = start.elapsed();
//         println!("Time elapsed: {:?}", duration);
//     }

//     async fn send_raw_transaction(
//         context: AppState,
//         rollup_id: String,
//         raw_transaction: RawTransaction,
//         count: u64,
//     ) -> Result<(), Error> {
//         let total_start = Instant::now();

//         let start: Instant = Instant::now();
//         let rollup = context.get_rollup(&rollup_id).await.unwrap();
//         let duration = start.elapsed();
//         tracing::info!("get_rollup {}: / {:?}", count, duration);

//         let start = Instant::now();
//         let rollup_metadata =
// context.get_rollup_metadata(&rollup_id).await.unwrap();         let duration
// = start.elapsed();         tracing::info!("get_rollup_metadata {}: / {:?}",
// count, duration);

//         let start = Instant::now();
//         let cluster = context
//             .get_cluster(
//                 rollup.platform,
//                 rollup.service_provider,
//                 &rollup.cluster_id,
//                 rollup_metadata.platform_block_height,
//             )
//             .await
//             .unwrap();
//         let duration = start.elapsed();
//         tracing::info!("get_cluster {}: / {:?}", count, duration);

//         let rollup_block_height = rollup_metadata.rollup_block_height;

//         let start = Instant::now();
//         let mut locked_rollup_metadata =
// context.get_mut_rollup_metadata(&rollup_id).await.unwrap();         let
// duration = start.elapsed();         tracing::info!("get_mut_rollup_metadata
// {}: / {:?}", count, duration);

//         let start = Instant::now();
//         let (transaction_order, pre_merkle_path) = locked_rollup_metadata
//
// .add_transaction_hash(raw_transaction.raw_transaction_hash().as_ref());
//         let duration = start.elapsed();
//         tracing::info!("add_transaction_hash {}: / {:?}", count, duration);

//         let start = Instant::now();
//         let order_commitment = issue_order_commitment(
//             context.clone(),
//             rollup.platform,
//             rollup_id.clone(),
//             rollup.order_commitment_type,
//             raw_transaction.raw_transaction_hash(),
//             rollup_block_height,
//             transaction_order,
//             pre_merkle_path,
//         )
//         .await
//         .unwrap();
//         let duration = start.elapsed();
//         tracing::info!("issue_order_commitment {}: / {:?}", count, duration);

//         let start = Instant::now();
//         let transaction_hash = raw_transaction.raw_transaction_hash();
//         let duration = start.elapsed();
//         tracing::info!("raw_transaction_hash {}: {:?}", count, duration);

//         let start = Instant::now();
//         RawTransactionModel::put_with_transaction_hash(
//             &rollup_id,
//             &transaction_hash,
//             raw_transaction.clone(),
//             true,
//         )
//         .unwrap();
//         let duration = start.elapsed();
//         tracing::info!("put_with_transaction_hash {}: {:?}", count,
// duration);

//         let start = Instant::now();
//         order_commitment
//             .put(&rollup_id, rollup_block_height, transaction_order)
//             .unwrap();
//         let duration = start.elapsed();
//         tracing::info!("order_commitment.put {}: {:?}", count, duration);

//         let duration = total_start.elapsed();
//         tracing::info!("send_raw_transaction - total {}: {:?}", count,
// duration);

//         Ok(())
//     }
// }
