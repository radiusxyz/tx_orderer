use std::str::FromStr;

use clap::{Parser, Subcommand};
use radius_sdk::{
    json_rpc::server::RpcServer,
    kvstore::{CachedKvStore, KvStore as Database},
    signature::{Address, PrivateKeySigner},
};
use sequencer::{
    client::{
        liveness::{
            self, distributed_key_generation::DistributedKeyGenerationClient, seeder::SeederClient,
        },
        validation,
    },
    error::{self, Error},
    logger::Logger,
    rpc::{
        cluster, external,
        internal::{self, GetSequencingInfo, GetSequencingInfos},
    },
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

    /// Register Sequencer: registerSequencer
    RegisterValidator {
        #[clap(flatten)]
        config_option: Box<ConfigRegisterValidator>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut cli = Cli::init();
    match cli.command {
        Commands::Init { ref config_path } => ConfigPath::init(config_path)?,
        Commands::Start {
            ref mut config_option,
        } => {
            // Load the configuration from the path
            let config = Config::load(config_option)?;

            // Initialize the logger.
            Logger::new(config.log_path())
                .map_err(error::Error::LoggerError)?
                .init();

            tracing::info!(
                "Successfully loaded the configuration file at {:?}.",
                config.path(),
            );

            // Initialize the database
            Database::new(config.database_path())
                .map_err(error::Error::Database)?
                .init();
            tracing::info!(
                "Successfully initialized the database at {:?}.",
                config.database_path(),
            );

            tracing::error!("Test - {:?}.", config.database_path(),);

            // Initialize seeder client
            let seeder_rpc_url = config.seeder_rpc_url();
            let seeder_client = SeederClient::new(seeder_rpc_url)?;
            tracing::info!(
                "Successfully initialized seeder client {:?}.",
                seeder_rpc_url,
            );

            // Initialize distributed key generation client
            let distributed_key_generation_rpc_url = config.distributed_key_generation_rpc_url();
            let distributed_key_generation_client =
                DistributedKeyGenerationClient::new(distributed_key_generation_rpc_url)?;

            let signing_key = config.signing_key();
            let signers = CachedKvStore::default();
            let liveness_clients = CachedKvStore::default();
            let validation_clients = CachedKvStore::default();

            // Initialize liveness clients
            let sequencing_info_list =
                SequencingInfoList::get_or(SequencingInfoList::default).map_err(Error::Database)?;
            for (platform, service_provider) in sequencing_info_list.iter() {
                tracing::info!(
                    "Initialize sequencing info - platform: {:?}, service_provider: {:?}",
                    platform,
                    service_provider
                );

                // Initialize the signer
                let signer = PrivateKeySigner::from_str((*platform).into(), signing_key)
                    .map_err(Error::Signature)?;
                signers
                    .put(platform, signer)
                    .await
                    .map_err(Error::CachedKvStore)?;

                let sequencing_info_payload =
                    SequencingInfoPayload::get(*platform, *service_provider)
                        .map_err(Error::Database)?;

                match sequencing_info_payload {
                    SequencingInfoPayload::Ethereum(liveness_info) => {
                        tracing::info!(
                            "Initialize liveness client - platform: {:?}, service_provider: {:?}",
                            platform,
                            service_provider
                        );

                        let liveness_client = liveness::radius::LivenessClient::new(
                            *platform,
                            *service_provider,
                            liveness_info,
                            config.signing_key(),
                            seeder_client.clone(),
                        )?;

                        liveness_client.initialize_event_listener();
                        let current_block = liveness_client
                            .publisher()
                            .get_block_number()
                            .await
                            .unwrap();
                        let block_margin: u64 = liveness_client
                            .publisher()
                            .get_block_margin()
                            .await
                            .unwrap()
                            .try_into()
                            .unwrap();

                        // Get the cluster ID list for a given liveness client.
                        let cluster_id_list = ClusterIdList::get_or(
                            liveness_client.platform(),
                            liveness_client.service_provider(),
                            ClusterIdList::default,
                        )
                        .unwrap();

                        for platform_block_height in (current_block - block_margin)..current_block {
                            for cluster_id in cluster_id_list.iter() {
                                let mut my_index: Option<usize> = None;

                                // Get the sequencer address list for the cluster ID.
                                let sequencer_address_list: Vec<String> = liveness_client
                                    .publisher()
                                    .get_sequencer_list(cluster_id, platform_block_height)
                                    .await
                                    .unwrap()
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, address)| {
                                        if address == liveness_client.publisher().address() {
                                            my_index = Some(index);
                                        }

                                        address.to_string()
                                    })
                                    .collect();

                                // Get the rollup info list
                                let rollup_info_list = liveness_client
                                    .publisher()
                                    .get_rollup_info_list(cluster_id, platform_block_height)
                                    .await
                                    .unwrap();

                                let rollup_id_list = rollup_info_list
                                    .iter()
                                    .map(|rollup_info| rollup_info.rollupId.clone())
                                    .collect();

                                // Update the rollup info to database
                                for rollup_info in rollup_info_list {
                                    match Rollup::get_mut(&rollup_info.rollupId) {
                                        Ok(mut rollup) => {
                                            let new_executor_address_list = rollup_info
                                                .executorAddresses
                                                .into_iter()
                                                .map(|address| {
                                                    Address::from_str(
                                                        Platform::from_str(
                                                            &rollup_info.validationInfo.platform,
                                                        )
                                                        .unwrap()
                                                        .into(),
                                                        &address.to_string(),
                                                    )
                                                    .unwrap()
                                                })
                                                .collect::<Vec<Address>>();
                                            rollup.set_executor_address_list(
                                                new_executor_address_list,
                                            );
                                            rollup.update().unwrap();
                                        }
                                        Err(error) => {
                                            if error.is_none_type() {
                                                let order_commitment_type =
                                                    OrderCommitmentType::from_str(
                                                        &rollup_info.orderCommitmentType,
                                                    )
                                                    .unwrap();
                                                let rollup_type =
                                                    RollupType::from_str(&rollup_info.rollupType)
                                                        .unwrap();
                                                let validation_info = ValidationInfo::new(
                                                    Platform::from_str(
                                                        &rollup_info.validationInfo.platform,
                                                    )
                                                    .unwrap(),
                                                    ValidationServiceProvider::from_str(
                                                        &rollup_info.validationInfo.serviceProvider,
                                                    )
                                                    .unwrap(),
                                                );
                                                let executor_address_list = rollup_info
                                                    .executorAddresses
                                                    .into_iter()
                                                    .map(|address| {
                                                        Address::from_str(
                                                            Platform::from_str(
                                                                &rollup_info
                                                                    .validationInfo
                                                                    .platform,
                                                            )
                                                            .unwrap()
                                                            .into(),
                                                            &address.to_string(),
                                                        )
                                                        .unwrap()
                                                    })
                                                    .collect::<Vec<Address>>();

                                                let rollup = Rollup::new(
                                                    rollup_info.rollupId.clone(),
                                                    rollup_type,
                                                    EncryptedTransactionType::Skde,
                                                    rollup_info.owner.to_string(),
                                                    validation_info,
                                                    order_commitment_type,
                                                    executor_address_list,
                                                    cluster_id.to_owned(),
                                                    liveness_client.platform(),
                                                    liveness_client.service_provider(),
                                                );

                                                Rollup::put(&rollup, rollup.rollup_id()).unwrap();

                                                // let rollup_metadata =
                                                // RollupMetadata::default();
                                                // RollupMetadataModel::put(rollup.rollup_id(),
                                                // &rollup_metadata)
                                                //     .unwrap();
                                            }
                                        }
                                    }
                                }

                                let sequencer_rpc_url_list = liveness_client
                                    .seeder()
                                    .get_sequencer_rpc_url_list(sequencer_address_list.clone())
                                    .await
                                    .unwrap()
                                    .sequencer_rpc_url_list;

                                let cluster = Cluster::new(
                                    sequencer_rpc_url_list,
                                    rollup_id_list,
                                    my_index.unwrap(),
                                    block_margin.try_into().unwrap(),
                                );

                                Cluster::put_and_update_with_margin(
                                    &cluster,
                                    liveness_client.platform(),
                                    liveness_client.service_provider(),
                                    cluster_id,
                                    platform_block_height,
                                )
                                .unwrap();
                            }
                        }

                        liveness_clients
                            .put(&(*platform, *service_provider), liveness_client)
                            .await
                            .map_err(Error::CachedKvStore)?;
                    }
                    SequencingInfoPayload::Local(_payload) => {
                        // liveness::local::LivenessClient::new()?;
                        todo!("Implement 'LivenessClient' for local sequencing.");
                    }
                }
            }

            // Initialize validation clients
            let validation_info_list =
                ValidationInfoList::get_or(ValidationInfoList::default).map_err(Error::Database)?;
            for (platform, service_provider) in validation_info_list.iter() {
                let validation_info_payload =
                    ValidationInfoPayload::get(*platform, *service_provider)
                        .map_err(Error::Database)?;

                match validation_info_payload {
                    ValidationInfoPayload::EigenLayer(validation_info) => {
                        let validation_client = validation::eigenlayer::ValidationClient::new(
                            *platform,
                            *service_provider,
                            validation_info,
                            signing_key,
                        )?;
                        validation_client.initialize_event_listener();

                        validation_clients
                            .put(&(*platform, *service_provider), validation_client)
                            .await
                            .map_err(Error::CachedKvStore)?;
                    }
                    ValidationInfoPayload::Symbiotic(validation_info) => {
                        let validation_client = validation::symbiotic::ValidationClient::new(
                            *platform,
                            *service_provider,
                            validation_info,
                            signing_key,
                        )?;
                        validation_client.initialize_event_listener();

                        validation_clients
                            .put(&(*platform, *service_provider), validation_client)
                            .await
                            .map_err(Error::CachedKvStore)?;
                    }
                }
            }

            let skde_params = distributed_key_generation_client
                .get_skde_params()
                .await?
                .skde_params;

            // Initialize an application-wide state instance
            let app_state = AppState::new(
                config,
                seeder_client,
                distributed_key_generation_client,
                signers,
                liveness_clients,
                validation_clients,
                skde_params,
            );

            // Initialize the internal RPC server
            initialize_internal_rpc_server(&app_state).await?;

            // Initialize the cluster RPC server
            initialize_cluster_rpc_server(&app_state).await?;

            // Initialize the external RPC server.
            let server_handle = initialize_external_rpc_server(&app_state).await?;

            server_handle.await.unwrap();
        }

        Commands::RegisterValidator { config_option } => {
            config_option.init().await;
        }
    }

    Ok(())
}

async fn initialize_internal_rpc_server(context: &AppState) -> Result<(), Error> {
    let internal_rpc_url = context.config().internal_rpc_url().to_string();

    // Initialize the internal RPC server.
    let internal_rpc_server = RpcServer::new(context.clone())
        .register_rpc_method(
            internal::AddSequencingInfo::METHOD_NAME,
            internal::AddSequencingInfo::handler,
        )?
        .register_rpc_method(
            internal::AddValidationInfo::METHOD_NAME,
            internal::AddValidationInfo::handler,
        )?
        .register_rpc_method(
            internal::AddCluster::METHOD_NAME,
            internal::AddCluster::handler,
        )?
        .register_rpc_method(
            internal::debug::GetCluster::METHOD_NAME,
            internal::debug::GetCluster::handler,
        )?
        .register_rpc_method(
            internal::debug::GetClusterIdList::METHOD_NAME,
            internal::debug::GetClusterIdList::handler,
        )?
        .register_rpc_method(GetSequencingInfos::METHOD_NAME, GetSequencingInfos::handler)?
        .register_rpc_method(GetSequencingInfo::METHOD_NAME, GetSequencingInfo::handler)?
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

async fn initialize_cluster_rpc_server(context: &AppState) -> Result<(), Error> {
    let cluster_rpc_url = anywhere(&context.config().cluster_port()?);

    let sequencer_rpc_server = RpcServer::new(context.clone())
        .register_rpc_method(
            cluster::SyncEncryptedTransaction::METHOD_NAME,
            cluster::SyncEncryptedTransaction::handler,
        )?
        .register_rpc_method(
            cluster::SyncRawTransaction::METHOD_NAME,
            cluster::SyncRawTransaction::handler,
        )?
        .register_rpc_method(
            cluster::FinalizeBlock::METHOD_NAME,
            cluster::FinalizeBlock::handler,
        )?
        .register_rpc_method(cluster::SyncBlock::METHOD_NAME, cluster::SyncBlock::handler)?
        .register_rpc_method(
            external::GetRawTransactionList::METHOD_NAME,
            external::GetRawTransactionList::handler,
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

async fn initialize_external_rpc_server(context: &AppState) -> Result<JoinHandle<()>, Error> {
    let external_rpc_url = anywhere(&context.config().external_port()?);

    // Initialize the external RPC server.
    let external_rpc_server = RpcServer::new(context.clone())
        .register_rpc_method(
            external::SendEncryptedTransaction::METHOD_NAME,
            external::SendEncryptedTransaction::handler,
        )?
        .register_rpc_method(
            external::GetEncryptedTransactionWithTransactionHash::METHOD_NAME,
            external::GetEncryptedTransactionWithTransactionHash::handler,
        )?
        .register_rpc_method(
            external::GetEncryptedTransactionWithOrderCommitment::METHOD_NAME,
            external::GetEncryptedTransactionWithOrderCommitment::handler,
        )?
        .register_rpc_method(
            external::GetRawTransactionWithTransactionHash::METHOD_NAME,
            external::GetRawTransactionWithTransactionHash::handler,
        )?
        .register_rpc_method(
            external::GetRawTransactionWithOrderCommitment::METHOD_NAME,
            external::GetRawTransactionWithOrderCommitment::handler,
        )?
        .register_rpc_method(
            external::SendRawTransaction::METHOD_NAME,
            external::SendRawTransaction::handler,
        )?
        .register_rpc_method(
            external::GetRawTransactionList::METHOD_NAME,
            external::GetRawTransactionList::handler,
        )?
        .register_rpc_method(
            internal::debug::GetRollup::METHOD_NAME,
            internal::debug::GetRollup::handler,
        )?
        .register_rpc_method(external::GetBlock::METHOD_NAME, external::GetBlock::handler)?
        .init(external_rpc_url.clone())
        .await?;

    tracing::info!(
        "Successfully started the sequencer external RPC server: {}",
        external_rpc_url
    );

    let server_handle = tokio::spawn(async move {
        external_rpc_server.stopped().await;
    });

    Ok(server_handle)
}

pub fn anywhere(port: &str) -> String {
    format!("0.0.0.0:{}", port)
}
