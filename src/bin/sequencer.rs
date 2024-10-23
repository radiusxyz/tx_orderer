use std::{fs, path::Path};

use clap::{Parser, Subcommand};
use pvde::{
    encryption::poseidon_encryption_zkp::{
        export_proving_key as export_poseidon_encryption_proving_key,
        export_verifying_key as export_poseidon_encryption_verifying_key,
        export_zkp_param as export_poseidon_encryption_zkp_param,
        import_proving_key as import_poseidon_encryption_proving_key,
        import_verifying_key as import_poseidon_encryption_verifying_key,
        import_zkp_param as import_poseidon_encryption_zkp_param,
        setup as setup_poseidon_encryption,
    },
    time_lock_puzzle::{
        export_time_lock_puzzle_param, import_time_lock_puzzle_param,
        key_validation_zkp::{
            export_proving_key as export_key_validation_proving_key,
            export_verifying_key as export_key_validation_verifying_key,
            export_zkp_param as export_key_validation_zkp_param,
            import_proving_key as import_key_validation_proving_key,
            import_verifying_key as import_key_validation_verifying_key,
            import_zkp_param as import_key_validation_zkp_param, setup as setup_key_validation,
        },
        setup as setup_time_lock_puzzle_param,
    },
};
use radius_sdk::{
    json_rpc::server::RpcServer,
    kvstore::{CachedKvStore, KvStore as Database},
    signature::PrivateKeySigner,
};
use sequencer::{
    client::{
        liveness::{self, key_management_system::KeyManagementSystemClient, seeder::SeederClient},
        validation,
    },
    error::{self, Error},
    rpc::{
        cluster, external,
        internal::{self, GetSequencingInfo, GetSequencingInfos},
    },
    state::AppState,
    types::*,
};
pub use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tracing::info;

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
    tracing_subscriber::fmt().init();
    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("{:?}", panic_info);
    }));

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
            Database::new(config.database_path())
                .map_err(error::Error::Database)?
                .init();
            tracing::info!(
                "Successfully initialized the database at {:?}.",
                config.database_path(),
            );

            // Initialize seeder client
            let seeder_rpc_url = config.seeder_rpc_url();
            let seeder_client = SeederClient::new(seeder_rpc_url)?;
            tracing::info!(
                "Successfully initialized seeder client {:?}.",
                seeder_rpc_url,
            );

            // Initialize key management system client
            let key_management_system_rpc_url = config.key_management_system_rpc_url();
            let key_management_system_client =
                KeyManagementSystemClient::new(key_management_system_rpc_url)?;

            let signing_key = config.signing_key();
            let signers = CachedKvStore::default();
            let liveness_clients = CachedKvStore::default();
            let validation_clients = CachedKvStore::default();

            // Initialize liveness clients
            let sequencing_info_list =
                SequencingInfoList::get_or(SequencingInfoList::default).map_err(Error::Database)?;
            for (platform, service_provider) in sequencing_info_list.iter() {
                info!(
                    "Initialize sequencing info - platform: {:?}, service_provider: {:?}",
                    platform, service_provider
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
                        let liveness_client = liveness::radius::LivenessClient::new(
                            *platform,
                            *service_provider,
                            liveness_info,
                            config.signing_key(),
                            seeder_client.clone(),
                        )?;
                        liveness_client.initialize_event_listener();

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

            // TODO: PVDE
            let path = config_option.path.clone().unwrap();
            let pvde_params = init_time_lock_puzzle_param(&path)?;

            let skde_params = key_management_system_client
                .get_skde_params()
                .await?
                .skde_params;

            // Initialize an application-wide state instance
            let app_state = AppState::new(
                config,
                seeder_client,
                key_management_system_client,
                signers,
                liveness_clients,
                validation_clients,
                pvde_params,
                skde_params,
            );

            // Initialize the internal RPC server
            initialize_internal_rpc_server(&app_state).await?;

            // TODO - check cluster rpc list
            // // Initialize the cluster RPC server
            // initialize_cluster_rpc_server(&app_state).await?;

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

async fn _initialize_cluster_rpc_server(context: &AppState) -> Result<(), Error> {
    let cluster_rpc_url = context.config().cluster_rpc_url().to_string();

    let sequencer_rpc_server = RpcServer::new(context.clone())
        .register_rpc_method(
            cluster::SyncEncryptedTransaction::METHOD_NAME,
            cluster::SyncEncryptedTransaction::handler,
        )?
        .register_rpc_method(
            cluster::SyncRawTransaction::METHOD_NAME,
            cluster::SyncRawTransaction::handler,
        )?
        // TODO:
        .register_rpc_method(
            cluster::FinalizeBlock::METHOD_NAME,
            cluster::FinalizeBlock::handler,
        )?
        // TODO:
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
    let external_rpc_url = context.config().external_rpc_url().to_string();

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
            cluster::FinalizeBlock::METHOD_NAME,
            cluster::FinalizeBlock::handler,
        )?
        .register_rpc_method(
            internal::debug::GetRollup::METHOD_NAME,
            internal::debug::GetRollup::handler,
        )?
        // cluster
        .register_rpc_method(
            cluster::SyncEncryptedTransaction::METHOD_NAME,
            cluster::SyncEncryptedTransaction::handler,
        )?
        .register_rpc_method(
            cluster::SyncRawTransaction::METHOD_NAME,
            cluster::SyncRawTransaction::handler,
        )?
        .register_rpc_method(cluster::SyncBlock::METHOD_NAME, cluster::SyncBlock::handler)?
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

pub fn init_time_lock_puzzle_param(config_path: &Path) -> Result<PvdeParams, Error> {
    let time_lock_puzzle_param_path = config_path
        .join("time_lock_puzzle_param.json")
        .to_str()
        .unwrap()
        .to_string();

    let time_lock_puzzle_param = if fs::metadata(&time_lock_puzzle_param_path).is_ok() {
        import_time_lock_puzzle_param(&time_lock_puzzle_param_path)
    } else {
        let time_lock_puzzle_param = setup_time_lock_puzzle_param(2048);
        export_time_lock_puzzle_param(&time_lock_puzzle_param_path, time_lock_puzzle_param.clone());
        time_lock_puzzle_param
    };

    let mut pvde_params = PvdeParams::default();
    pvde_params.update_time_lock_puzzle_param(time_lock_puzzle_param);

    let key_validation_param_file_path = config_path
        .join("key_validation_zkp_param.data")
        .to_str()
        .unwrap()
        .to_string();
    let key_validation_proving_key_file_path = config_path
        .join("key_validation_proving_key.data")
        .to_str()
        .unwrap()
        .to_string();
    let key_validation_verifying_key_file_path = config_path
        .join("key_validation_verifying_key.data")
        .to_str()
        .unwrap()
        .to_string();

    let (key_validation_zkp_param, key_validation_verifying_key, key_validation_proving_key) =
        if fs::metadata(&key_validation_param_file_path).is_ok() {
            (
                import_key_validation_zkp_param(&key_validation_param_file_path),
                import_key_validation_verifying_key(&key_validation_verifying_key_file_path),
                import_key_validation_proving_key(&key_validation_proving_key_file_path),
            )
        } else {
            let setup_results = setup_key_validation(13);
            export_key_validation_zkp_param(
                &key_validation_param_file_path,
                setup_results.0.clone(),
            );
            export_key_validation_verifying_key(
                &key_validation_verifying_key_file_path,
                setup_results.1.clone(),
            );
            export_key_validation_proving_key(
                &key_validation_proving_key_file_path,
                setup_results.2.clone(),
            );
            setup_results
        };

    pvde_params.update_key_validation_zkp_param(key_validation_zkp_param);
    pvde_params.update_key_validation_proving_key(key_validation_proving_key);
    pvde_params.update_key_validation_verifying_key(key_validation_verifying_key);

    let poseidon_encryption_param_file_path = config_path
        .join("poseidon_encryption_param.json")
        .to_str()
        .unwrap()
        .to_string();
    let poseidon_encryption_proving_key_file_path = config_path
        .join("poseidon_encryption_proving_key.data")
        .to_str()
        .unwrap()
        .to_string();
    let poseidon_encryption_verifying_key_file_path = config_path
        .join("poseidon_encryption_verifying_key.data")
        .to_str()
        .unwrap()
        .to_string();

    let (
        poseidon_encryption_zkp_param,
        poseidon_encryption_verifying_key,
        poseidon_encryption_proving_key,
    ) = if fs::metadata(&poseidon_encryption_param_file_path).is_ok() {
        (
            import_poseidon_encryption_zkp_param(&poseidon_encryption_param_file_path),
            import_poseidon_encryption_verifying_key(&poseidon_encryption_verifying_key_file_path),
            import_poseidon_encryption_proving_key(&poseidon_encryption_proving_key_file_path),
        )
    } else {
        let setup_results = setup_poseidon_encryption(13);
        export_poseidon_encryption_zkp_param(
            &poseidon_encryption_param_file_path,
            setup_results.0.clone(),
        );
        export_poseidon_encryption_verifying_key(
            &poseidon_encryption_verifying_key_file_path,
            setup_results.1.clone(),
        );
        export_poseidon_encryption_proving_key(
            &poseidon_encryption_proving_key_file_path,
            setup_results.2.clone(),
        );
        setup_results
    };

    pvde_params.update_poseidon_encryption_zkp_param(poseidon_encryption_zkp_param);
    pvde_params.update_poseidon_encryption_proving_key(poseidon_encryption_proving_key);
    pvde_params.update_poseidon_encryption_verifying_key(poseidon_encryption_verifying_key);

    Ok(pvde_params)
}
