use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
    sync::Arc,
};

use radius_sdk::{
    liveness::radius::{
        publisher::Publisher,
        subscriber::Subscriber,
        types::{Events, ILivenessRadius::Rollup as RollupInfo},
    },
    signature::{Address, PrivateKeySigner},
};
use tokio::time::{sleep, Duration};

use super::seeder::SequencerRpcInfo;
use crate::{
    client::liveness_service_manager::seeder::SeederClient, error::Error, state::AppState, types::*,
};

pub struct LivenessClient {
    inner: Arc<LivenessClientInner>,
}

struct LivenessClientInner {
    platform: Platform,
    service_provider: ServiceProvider,
    publisher: Publisher,
    subscriber: Subscriber,
    seeder: SeederClient,
}

impl Clone for LivenessClient {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl LivenessClient {
    pub fn new(
        platform: Platform,
        service_provider: ServiceProvider,
        liveness_info: LivenessRadius,
        signing_key: impl AsRef<str>,
        seeder: SeederClient,
    ) -> Result<Self, Error> {
        let publisher = Publisher::new(
            liveness_info.liveness_rpc_url,
            signing_key,
            &liveness_info.contract_address,
        )
        .map_err(|error| Error::LivenessClient(error.into()))?;

        let subscriber = Subscriber::new(
            liveness_info.liveness_websocket_url,
            liveness_info.contract_address,
        )
        .map_err(|error| Error::LivenessClient(error.into()))?;

        Ok(Self {
            inner: Arc::new(LivenessClientInner {
                platform,
                service_provider,
                publisher,
                subscriber,
                seeder,
            }),
        })
    }

    pub fn initialize(
        context: AppState,
        platform: Platform,
        service_provider: ServiceProvider,
        liveness_info: LivenessRadius,
    ) {
        let handle = tokio::spawn({
            let context_clone = context.clone();
            let liveness_info_clone = liveness_info.clone();

            async move {
                let signing_key = &context_clone.config().signing_key;
                let signer = PrivateKeySigner::from_str(platform.into(), signing_key)
                    .expect("Invalid signing key");

                context_clone
                    .add_signer(platform, signer)
                    .await
                    .expect("Failed to add signer");

                let liveness_client = Self::new(
                    platform,
                    service_provider,
                    liveness_info_clone,
                    signing_key,
                    context_clone.seeder_client().clone(),
                )
                .expect("Failed to create liveness client");

                let current_block_height = liveness_client
                    .publisher()
                    .get_block_number()
                    .await
                    .expect("Failed to get block number");

                let block_margin: u64 = liveness_client
                    .publisher()
                    .get_block_margin()
                    .await
                    .expect("Failed to get block margin")
                    .try_into()
                    .expect("Failed to convert block margin");

                let cluster_id_list = ClusterIdList::get_or(
                    liveness_client.platform(),
                    liveness_client.service_provider(),
                    ClusterIdList::default,
                )
                .expect("Failed to get cluster id list");

                for cluster_id in cluster_id_list.iter() {
                    initialize_new_cluster(
                        context_clone.clone(),
                        &liveness_client,
                        cluster_id,
                        current_block_height,
                        block_margin,
                    )
                    .await
                    .unwrap();
                }

                context_clone
                    .add_liveness_client(platform, service_provider, liveness_client.clone())
                    .await
                    .expect("Failed to add liveness client");

                tracing::info!(
                    "Initializing the liveness event listener for {:?}, {:?}..",
                    platform,
                    service_provider
                );

                let result = liveness_client
                    .subscriber()
                    .initialize_event_handler(callback, (context_clone, liveness_client.clone()))
                    .await;

                if let Err(error) = result {
                    tracing::error!(
                        "Failed to initialize the liveness event listener for {:?}, {:?} - {:?}",
                        platform,
                        service_provider,
                        error
                    );
                }
            }
        });

        tokio::spawn(async move {
            if handle.await.is_err() {
                tracing::warn!(
                    "Reconnecting the liveness event listener for {:?}, {:?}..",
                    platform,
                    service_provider
                );
                sleep(Duration::from_secs(5)).await;
                Self::initialize(context, platform, service_provider, liveness_info);
            }
        });
    }
}

async fn callback(events: Events, (app_state, liveness_client): (AppState, LivenessClient)) {
    tracing::debug!(
        "Received a new event - platform: {:?} / service provider: {:?}..",
        liveness_client.platform(),
        liveness_client.service_provider()
    );

    match events {
        Events::Block(block) => {
            tracing::debug!(
                "Received a new block - platform: {:?} / service provider: {:?} / block number: {:?}..",
                liveness_client.platform(),
                liveness_client.service_provider(),
                block.number
            );

            let cluster_id_list = ClusterIdList::get_or(
                liveness_client.platform(),
                liveness_client.service_provider(),
                ClusterIdList::default,
            )
            .expect("Failed to get cluster id list");

            let block_margin = liveness_client
                .publisher()
                .get_block_margin()
                .await
                .expect("Failed to get block margin")
                .try_into()
                .expect("Failed to convert block margin");

            for cluster_id in cluster_id_list.iter() {
                initialize_new_cluster(
                    app_state.clone(),
                    &liveness_client,
                    cluster_id,
                    block.number,
                    block_margin,
                )
                .await
                .expect("Failed to initialize new cluster");
            }
        }
        _others => {}
    }
}

pub async fn initialize_new_cluster(
    context: AppState,
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
    block_margin: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!(
        "Initializing the cluster - platform: {:?} / service provider: {:?} / cluster id: {:?} / platform_block_height: {:?}..",
        liveness_client.platform(),
        liveness_client.service_provider(),
        cluster_id,
        platform_block_height
    );

    let mut latest_cluster_block_height = LatestClusterBlockHeight::get_mut_or(
        liveness_client.platform(),
        liveness_client.service_provider(),
        cluster_id,
        LatestClusterBlockHeight::default,
    )?;

    let block_diff = platform_block_height - latest_cluster_block_height.get_block_height();
    let block_diff = std::cmp::min(block_diff, block_margin);

    for offset in 0..block_diff {
        let mut retries = 5;
        while retries > 0 {
            let block_height = platform_block_height - offset;
            tracing::debug!(
                "Sync the cluster - platform: {:?} / service provider: {:?} / cluster id: {:?} / block height: {:?}..",
                liveness_client.platform(),
                liveness_client.service_provider(),
                cluster_id,
                block_height
            );
            match get_sequencer_rpc_infos(&liveness_client, cluster_id, block_height).await {
                Ok(sequencer_rpc_infos) => {
                    let rollup_id_list =
                        get_rollup_id_list(&liveness_client, cluster_id, block_height).await?;

                    let sequencer_address = context
                        .get_signer(liveness_client.platform())
                        .await?
                        .address()
                        .clone();

                    let cluster = Cluster::new(
                        sequencer_rpc_infos,
                        rollup_id_list,
                        sequencer_address,
                        block_margin,
                    );
                    cluster.put(
                        liveness_client.platform(),
                        liveness_client.service_provider(),
                        cluster_id,
                        block_height,
                    )?;

                    tracing::debug!(
                        "Sync the cluster - platform: {:?} / service provider: {:?} / cluster id: {:?} / block height: {:?} - Done",
                        liveness_client.platform(),
                        liveness_client.service_provider(),
                        cluster_id,
                        block_height
                    );

                    break;
                }
                Err(e) => {
                    retries -= 1;
                    tracing::warn!(
                        "Failed to fetch sequencer RPC infos for cluster: {}, height: {}, error: {:?} (remaining retries: {})",
                        cluster_id,
                        block_height,
                        e,
                        retries
                    );

                    if retries == 0 {
                        return Err(e.into());
                    }

                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        sleep(Duration::from_millis(500)).await;
    }

    latest_cluster_block_height.set_block_height(platform_block_height);
    latest_cluster_block_height.update()?;

    tracing::debug!(
        "Initializing the cluster - platform: {:?} / service provider: {:?} / cluster id: {:?} / platform_block_height: {:?} - Done",
        liveness_client.platform(),
        liveness_client.service_provider(),
        cluster_id,
        platform_block_height
    );

    Ok(())
}

async fn get_sequencer_rpc_infos(
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
) -> Result<BTreeMap<usize, SequencerRpcInfo>, Error> {
    let sequencer_address_list = liveness_client
        .publisher()
        .get_sequencer_list(cluster_id, platform_block_height)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to fetch sequencer list for cluster: {}, height: {}. Error: {:?}",
                cluster_id,
                platform_block_height,
                e
            );
            e
        })
        .map_err(|e| Error::LivenessClient(e.into()))?
        .into_iter()
        .map(|a| a.to_string())
        .collect();

    let sequencer_rpc_url_list = liveness_client
        .seeder()
        .get_sequencer_rpc_url_list(sequencer_address_list)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to fetch sequencer RPC URLs for cluster: {}, height: {}. Error: {:?}",
                cluster_id,
                platform_block_height,
                e
            );
            e
        })?
        .sequencer_rpc_url_list;

    let sequencer_rpc_infos = sequencer_rpc_url_list
        .into_iter()
        .enumerate()
        .collect::<BTreeMap<usize, SequencerRpcInfo>>();

    Ok(sequencer_rpc_infos)
}

async fn get_rollup_id_list(
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let rollup_list = liveness_client
        .publisher()
        .get_rollup_info_list(cluster_id, platform_block_height)
        .await?;

    for rollup in rollup_list.iter() {
        let validation_service_provider = ValidationServiceProvider::from_str(
            &rollup.validationInfo.serviceProvider,
        )
        .expect(&format!(
            "Unsupported validation service provider: {:?}",
            &rollup.validationInfo.serviceProvider
        ));

        update_or_create_rollup(
            liveness_client.platform(),
            liveness_client.service_provider(),
            validation_service_provider,
            cluster_id,
            rollup,
        )
        .await?;
    }

    Ok(rollup_list.iter().map(|rollup| rollup.id.clone()).collect())
}

async fn update_or_create_rollup(
    platform: Platform,
    service_provider: ServiceProvider,
    validation_service_provider: ValidationServiceProvider,
    cluster_id: &str,
    rollup_info: &RollupInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    match Rollup::get_mut(&rollup_info.id) {
        Ok(mut rollup) => {
            let executor_address_list = rollup_info
                .executors
                .iter()
                .map(|addr| address_from_str(platform, addr.to_string()))
                .collect();
            rollup.set_executor_address_list(executor_address_list);
            rollup.update()?;

            Ok(())
        }
        Err(error) => {
            if error.is_none_type() {
                let validation_service_manager_address = address_from_str(
                    platform,
                    rollup_info
                        .validationInfo
                        .validationServiceManager
                        .to_string(),
                );

                let rollup_validation_info = RollupValidationInfo::new(
                    platform,
                    validation_service_provider,
                    validation_service_manager_address,
                );

                let executor_address_list = rollup_info
                    .executors
                    .iter()
                    .map(|addr| address_from_str(platform, addr.to_string()))
                    .collect();

                let rollup_type = RollupType::from_str(&rollup_info.rollupType).expect(&format!(
                    "Unsupported rollup type: {:?}",
                    &rollup_info.rollupType
                ));

                let order_commitment_type = OrderCommitmentType::from_str(
                    &rollup_info.orderCommitmentType,
                )
                .expect(&format!(
                    "Unsupported order commitment type: {:?}",
                    &rollup_info.orderCommitmentType
                ));

                let rollup = Rollup::new(
                    rollup_info.id.clone(),
                    rollup_type,
                    EncryptedTransactionType::Skde,
                    address_from_str(platform, rollup_info.owner.to_string()),
                    rollup_validation_info,
                    order_commitment_type,
                    executor_address_list,
                    cluster_id.to_owned(),
                    platform,
                    service_provider,
                );

                let mut rollup_id_list = RollupIdList::get_mut_or(RollupIdList::default)?;
                rollup_id_list.insert(&rollup.rollup_id);
                rollup_id_list.update()?;

                let mut rollup_metadata = RollupMetadata::default();
                rollup_metadata.cluster_id = cluster_id.to_owned();
                rollup_metadata.put(&rollup.rollup_id)?;

                rollup.put(&rollup.rollup_id)?;

                Ok(())
            } else {
                return Err(error.into());
            }
        }
    }
}

fn address_from_str(platform: Platform, address: String) -> Address {
    Address::from_str(platform.into(), &address).expect("Invalid address")
}

impl LivenessClient {
    pub fn platform(&self) -> Platform {
        self.inner.platform
    }

    pub fn service_provider(&self) -> ServiceProvider {
        self.inner.service_provider
    }

    pub fn publisher(&self) -> &Publisher {
        &self.inner.publisher
    }

    pub fn subscriber(&self) -> &Subscriber {
        &self.inner.subscriber
    }

    pub fn seeder(&self) -> &SeederClient {
        &self.inner.seeder
    }
}
