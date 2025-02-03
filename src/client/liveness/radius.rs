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
use crate::{client::liveness::seeder::SeederClient, error::Error, state::AppState, types::*};

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
            inner: self.inner.clone(),
        }
    }
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

        let inner = LivenessClientInner {
            platform,
            service_provider,
            publisher,
            subscriber,
            seeder,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn initialize(
        context: AppState,
        platform: Platform,
        service_provider: ServiceProvider,
        liveness_info: LivenessRadius,
    ) {
        let handle = tokio::spawn({
            let context = context.clone();
            let liveness_info = liveness_info.clone();

            async move {
                let signing_key = &context.config().signing_key;
                let signer = PrivateKeySigner::from_str(platform.into(), signing_key).unwrap();
                context.add_signer(platform, signer).await.unwrap();

                let liveness_client = Self::new(
                    platform,
                    service_provider,
                    liveness_info,
                    signing_key,
                    context.seeder_client().clone(),
                )
                .unwrap();

                let current_block_height = liveness_client
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

                let cluster_id_list = ClusterIdList::get_or(
                    liveness_client.platform(),
                    liveness_client.service_provider(),
                    ClusterIdList::default,
                )
                .unwrap();

                for platform_block_height in
                    (current_block_height - block_margin)..current_block_height
                {
                    for cluster_id in cluster_id_list.iter() {
                        tracing::info!(
                            "Initializing the cluster - platform: {:?} / service provider: {:?} / cluster id: {:?} / platform_block_height: {:?}..",
                            liveness_client.platform(),
                            liveness_client.service_provider(),
                            cluster_id,
                            platform_block_height
                        );

                        initialize_new_cluster(
                            context.clone(),
                            liveness_client.clone(),
                            cluster_id,
                            platform_block_height,
                            block_margin,
                        )
                        .await
                        .unwrap();
                    }
                }

                context
                    .add_liveness_client(platform, service_provider, liveness_client.clone())
                    .await
                    .unwrap();

                tracing::info!(
                    "Initializing the liveness event listener for {:?}, {:?}..",
                    platform,
                    service_provider
                );
                liveness_client
                    .subscriber()
                    .initialize_event_handler(callback, (context, liveness_client.clone()))
                    .await
                    .unwrap();
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

async fn callback(events: Events, context: (AppState, LivenessClient)) {
    match events {
        Events::Block(block) => {
            let app_state = context.0;
            let liveness_client = context.1;

            let cluster_id_list = ClusterIdList::get_or(
                liveness_client.platform(),
                liveness_client.service_provider(),
                ClusterIdList::default,
            )
            .unwrap();

            let block_margin = liveness_client
                .publisher()
                .get_block_margin()
                .await
                .unwrap();

            for cluster_id in cluster_id_list.iter() {
                initialize_new_cluster(
                    app_state.clone(),
                    liveness_client.clone(),
                    cluster_id,
                    block.number,
                    block_margin.try_into().unwrap(),
                )
                .await
                .unwrap()
            }
        }
        _others => {}
    }
}

pub async fn initialize_new_cluster(
    context: AppState,
    liveness_client: LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
    block_margin: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let sequencer_rpc_infos =
        get_sequencer_rpc_infos(&liveness_client, cluster_id, platform_block_height).await?;

    let rollup_id_list = get_rollup_id_list(
        context.clone(),
        &liveness_client,
        cluster_id,
        platform_block_height,
    )
    .await?;

    let sequencer_address = context
        .get_signer(liveness_client.platform())
        .await?
        .address()
        .clone();

    let cluster = Cluster::new(
        sequencer_rpc_infos,
        rollup_id_list,
        sequencer_address,
        block_margin.try_into().unwrap(),
    );
    cluster.put(
        liveness_client.platform(),
        liveness_client.service_provider(),
        cluster_id,
        platform_block_height,
    )?;

    Ok(())
}

async fn get_sequencer_rpc_infos(
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
) -> Result<BTreeMap<usize, SequencerRpcInfo>, Box<dyn std::error::Error>> {
    let sequencer_address_list: Vec<String> = liveness_client
        .publisher()
        .get_sequencer_list(cluster_id, platform_block_height)
        .await?
        .iter()
        .map(|address| address.to_string())
        .collect();

    let sequencer_rpc_infos: BTreeMap<usize, SequencerRpcInfo> = liveness_client
        .seeder()
        .get_sequencer_rpc_url_list(sequencer_address_list)
        .await?
        .sequencer_rpc_url_list
        .into_iter()
        .enumerate()
        .map(|(index, rpc_info)| (index, rpc_info))
        .collect();

    Ok(sequencer_rpc_infos)
}

async fn get_rollup_id_list(
    context: AppState,
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let rollup_list = liveness_client
        .publisher()
        .get_rollup_info_list(cluster_id, platform_block_height)
        .await?;

    for rollup in rollup_list.iter() {
        tracing::info!("Initializing the rollup - rollup id: {:?}..", rollup.id,);
        let validation_service_provider =
            ValidationServiceProvider::from_str(&rollup.validationInfo.serviceProvider).unwrap();

        update_or_create_rollup(
            context.clone(),
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
    context: AppState,
    platform: Platform,
    service_provider: ServiceProvider,
    validation_service_provider: ValidationServiceProvider,
    cluster_id: &str,
    rollup_info: &RollupInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    let (rollup, rollup_metadata) = match Rollup::get_mut(&rollup_info.id) {
        Ok(mut rollup) => {
            let executor_address_list = rollup_info
                .executors
                .iter()
                .map(|addr| address_from_str(platform, addr.to_string()))
                .collect();
            rollup.set_executor_address_list(executor_address_list);
            let rollup_clone = rollup.clone();
            rollup.update()?;

            let rollup_metadata = RollupMetadata::get(&rollup_info.id)?;

            (rollup_clone, rollup_metadata)
        }
        Err(_) => {
            let validation_service_manager_address = address_from_str(
                platform,
                rollup_info
                    .validationInfo
                    .validationServiceManager
                    .to_string(),
            );

            let validation_info = RollupValidationInfo::new(
                platform,
                validation_service_provider,
                validation_service_manager_address,
            );

            let executor_address_list = rollup_info
                .executors
                .iter()
                .map(|addr| address_from_str(platform, addr.to_string()))
                .collect();

            let rollup = Rollup::new(
                rollup_info.id.clone(),
                RollupType::from_str(&rollup_info.rollupType).unwrap(),
                EncryptedTransactionType::Skde,
                address_from_str(platform, rollup_info.owner.to_string()),
                validation_info,
                OrderCommitmentType::from_str(&rollup_info.orderCommitmentType).unwrap(),
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

            Rollup::put(&rollup, &rollup.rollup_id)?;

            (rollup, rollup_metadata)
        }
    };

    context
        .add_rollup_metadata(&rollup.rollup_id, rollup_metadata)
        .await
        .unwrap();
    context
        .add_rollup(&rollup.rollup_id, rollup.clone())
        .await
        .unwrap();

    Ok(())
}

fn address_from_str(platform: Platform, address: String) -> Address {
    Address::from_str(platform.into(), &address).unwrap()
}
