use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
    sync::Arc,
};

use radius_sdk::{
    liveness::radius::{
        publisher::Publisher,
        subscriber::Subscriber,
        types::{
            rpc::types::Header, Events, ILivenessRadius::Rollup as RollupInfo,
            Liveness::LivenessEvents,
        },
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

                let cluster_id_list =
                    ClusterIdList::get_or(platform, service_provider, ClusterIdList::default)
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

                for platform_block_height in
                    (current_block_height - block_margin)..current_block_height
                {
                    for cluster_id in cluster_id_list.iter() {
                        tracing::info!(
                            "Initializing the cluster - platform: {:?} / service provider: {:?} / cluster id: {:?} / platform_block_height: {:?}..",
                            platform,
                            service_provider,
                            cluster_id,
                            platform_block_height
                        );

                        initialize_new_cluster(
                            context.clone(),
                            &liveness_client,
                            cluster_id,
                            platform_block_height,
                            block_margin,
                        )
                        .await;
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
        Events::Block(block) => on_new_block(context.0, block, context.1).await,
        Events::LivenessEvents(liveness_event, log) => match liveness_event {
            LivenessEvents::RegisteredSequencer(register_sequencer) => {
                let cluster_id = register_sequencer.clusterId;
                let platform_block_height = log.block_number.unwrap();
                let sequencer_index: usize = register_sequencer.index.try_into().unwrap();
                let sequencer_address = register_sequencer.sequencer.to_string();
                let sequencer_rpc_info = context
                    .1
                    .seeder()
                    .get_sequencer_rpc_url(sequencer_address.to_string())
                    .await
                    .unwrap()
                    .sequencer_rpc_url;

                let mut liveness_event_list = LivenessEventList::get_mut_or(
                    &cluster_id,
                    platform_block_height,
                    LivenessEventList::default,
                )
                .unwrap();

                liveness_event_list.push(LivenessEventType::RegisteredSequencer(
                    sequencer_index,
                    sequencer_rpc_info,
                ));

                liveness_event_list.update().unwrap();
            }
            LivenessEvents::DeregisteredSequencer(deregister_sequencer) => {
                let cluster_id = deregister_sequencer.clusterId;
                let platform_block_height = log.block_number.unwrap();
                let sequencer_address = deregister_sequencer.sequencer.to_string();

                let mut liveness_event_list = LivenessEventList::get_mut_or(
                    &cluster_id,
                    platform_block_height,
                    LivenessEventList::default,
                )
                .unwrap();
                liveness_event_list
                    .push(LivenessEventType::DeregisteredSequencer(sequencer_address));
                liveness_event_list.update().unwrap();
            }

            LivenessEvents::AddedRollup(added_rollup) => {
                let cluster_id = added_rollup.clusterId;
                let rollup_id = added_rollup.rollupId.to_string();

                let platform_block_height = log.block_number.unwrap();

                let mut liveness_event_list = LivenessEventList::get_mut_or(
                    &cluster_id,
                    platform_block_height,
                    LivenessEventList::default,
                )
                .unwrap();
                liveness_event_list.push(LivenessEventType::AddedRollup(cluster_id, rollup_id));
                liveness_event_list.update().unwrap();
            }
            _others => {}
        },
    }
}

async fn on_new_block(context: AppState, block: Header, liveness_client: LivenessClient) {
    let platform_block_height = block.number;
    let previous_block_height = platform_block_height.wrapping_sub(1);

    let cluster_id_list = ClusterIdList::get_or(
        liveness_client.platform(),
        liveness_client.service_provider(),
        ClusterIdList::default,
    )
    .unwrap();

    for cluster_id in cluster_id_list.iter() {
        if let Ok(cluster) = update_cluster(
            context.clone(),
            &liveness_client,
            cluster_id,
            previous_block_height,
        )
        .await
        {
            cluster
                .put(
                    liveness_client.platform(),
                    liveness_client.service_provider(),
                    cluster_id,
                    platform_block_height,
                )
                .unwrap();

            context
                .add_cluster(
                    liveness_client.platform(),
                    liveness_client.service_provider(),
                    cluster_id,
                    previous_block_height,
                    cluster,
                )
                .await
                .unwrap();
        } else {
            let block_margin: u64 = liveness_client
                .publisher()
                .get_block_margin()
                .await
                .unwrap()
                .try_into()
                .unwrap();

            initialize_new_cluster(
                context.clone(),
                &liveness_client,
                cluster_id,
                platform_block_height,
                block_margin,
            )
            .await;
        }
    }
}

async fn update_cluster(
    context: AppState,
    liveness_client: &LivenessClient,
    cluster_id: &str,
    previous_block_height: u64,
) -> Result<Cluster, Error> {
    let mut cluster = Cluster::get(
        liveness_client.platform(),
        liveness_client.service_provider(),
        cluster_id,
        previous_block_height,
    )
    .map_err(Error::Database)?;

    let liveness_event_list = LivenessEventList::get_or(
        cluster_id,
        previous_block_height,
        LivenessEventList::default,
    )
    .unwrap();

    let sequencer_address_list: Vec<String> = cluster
        .sequencer_rpc_infos
        .iter()
        .filter_map(|(_, sequencer_rpc_info)| {
            if sequencer_rpc_info.cluster_rpc_url.is_none() {
                return Some(sequencer_rpc_info.address.as_hex_string());
            }
            None
        })
        .collect();

    let sequencer_rpc_info_list = liveness_client
        .seeder()
        .get_sequencer_rpc_url_list(sequencer_address_list)
        .await
        .unwrap()
        .sequencer_rpc_url_list;

    cluster
        .sequencer_rpc_infos
        .iter_mut()
        .for_each(|(_, sequencer_rpc_info)| {
            if sequencer_rpc_info.cluster_rpc_url.is_none() {
                if let Some(rpc_info) = sequencer_rpc_info_list
                    .iter()
                    .find(|rpc_info| rpc_info.address == sequencer_rpc_info.address)
                {
                    *sequencer_rpc_info = rpc_info.clone();
                }
            }
        });

    for event in liveness_event_list.iter() {
        match event {
            LivenessEventType::RegisteredSequencer(sequencer_index, sequencer_rpc_info) => {
                cluster.register_sequencer(*sequencer_index, sequencer_rpc_info.clone());
            }
            LivenessEventType::DeregisteredSequencer(sequencer_address) => {
                cluster.deregister_sequencer(sequencer_address);
            }
            LivenessEventType::AddedRollup(cluster_id, rollup_id) => {
                let rollup_info = liveness_client
                    .publisher()
                    .get_rollup_info(&cluster_id, &rollup_id, previous_block_height)
                    .await
                    .unwrap();

                let validation_service_provider = ValidationServiceProvider::from_str(
                    &rollup_info.validationInfo.serviceProvider,
                )
                .unwrap();

                update_or_create_rollup(
                    context.clone(),
                    liveness_client.platform(),
                    liveness_client.service_provider(),
                    validation_service_provider,
                    &cluster_id,
                    &rollup_info,
                )
                .await
                .unwrap();

                cluster.add_rollup(rollup_id);
            }
        }
    }

    Ok(cluster)
}

pub async fn initialize_new_cluster(
    context: AppState,
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
    block_margin: u64,
) {
    match Cluster::get(
        liveness_client.platform(),
        liveness_client.service_provider(),
        cluster_id,
        platform_block_height,
    ) {
        Ok(cluster) => {
            context
                .add_cluster(
                    liveness_client.platform(),
                    liveness_client.service_provider(),
                    cluster_id,
                    platform_block_height,
                    cluster,
                )
                .await
                .unwrap();
        }
        Err(_) => {
            let sequencer_rpc_infos =
                fetch_sequencer_rpc_infos(liveness_client, cluster_id, platform_block_height).await;
            let rollup_id_list = initialize_rollups(
                context.clone(),
                liveness_client,
                cluster_id,
                platform_block_height,
            )
            .await;

            let sequencer_address = address_from_str(
                liveness_client.platform(),
                liveness_client.publisher().address().to_string(),
            );

            let cluster = Cluster::new(
                sequencer_rpc_infos,
                rollup_id_list,
                sequencer_address,
                block_margin,
            );

            Cluster::put_and_update_with_margin(
                context,
                &cluster,
                liveness_client.platform(),
                liveness_client.service_provider(),
                cluster_id,
                platform_block_height,
            )
            .await
            .unwrap();
        }
    }

    // let liveness_client = liveness_client.clone();
    // let context = context.clone();
    // let cluster_id = cluster_id.to_owned();
    // tokio::spawn(async move {
    //     let mut cluster = context
    //         .get_cluster(
    //             liveness_client.platform(),
    //             liveness_client.service_provider(),
    //             &cluster_id,
    //         )
    //         .await
    //         .unwrap();

    //     let mut sequencer_address_list = Vec::new();

    //     for (index, rpc_info) in cluster.sequencer_rpc_infos.iter() {
    //         if rpc_info.external_rpc_url.is_none() {
    //             continue;
    //         }
    //         match
    // health_check(&rpc_info.external_rpc_url.as_ref().unwrap()).await {
    //             Ok(_) => {}
    //             Err(_) => {
    //
    // sequencer_address_list.push(rpc_info.address.as_hex_string());
    //             }
    //         }
    //     }

    //     let sequencer_rpc_info_list = liveness_client
    //         .seeder()
    //         .get_sequencer_rpc_url_list(sequencer_address_list)
    //         .await
    //         .unwrap()
    //         .sequencer_rpc_url_list;

    //     for sequencer_rpc_info in sequencer_rpc_info_list.iter() {}
    // });
}

async fn fetch_sequencer_rpc_infos(
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
) -> BTreeMap<usize, SequencerRpcInfo> {
    let sequencer_address_list: Vec<String> = liveness_client
        .publisher()
        .get_sequencer_list(cluster_id, platform_block_height)
        .await
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(_, address)| address.to_string())
        .collect();

    let sequencer_rpc_info_list = liveness_client
        .seeder()
        .get_sequencer_rpc_url_list(sequencer_address_list)
        .await
        .unwrap()
        .sequencer_rpc_url_list;

    sequencer_rpc_info_list
        .into_iter()
        .enumerate()
        .map(|(index, rpc_info)| (index, rpc_info))
        .collect()
}

async fn initialize_rollups(
    context: AppState,
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
) -> BTreeSet<String> {
    let rollup_list = liveness_client
        .publisher()
        .get_rollup_info_list(cluster_id, platform_block_height)
        .await
        .unwrap();

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
        .await
        .unwrap();
    }

    rollup_list.iter().map(|rollup| rollup.id.clone()).collect()
}

async fn update_or_create_rollup(
    context: AppState,
    platform: Platform,
    service_provider: ServiceProvider,
    validation_service_provider: ValidationServiceProvider,
    cluster_id: &str,
    rollup_info: &RollupInfo,
) -> Result<(), Error> {
    let (rollup, rollup_metadata) = match Rollup::get_mut(&rollup_info.id) {
        Ok(mut rollup) => {
            let executor_address_list = rollup_info
                .executors
                .iter()
                .map(|addr| address_from_str(platform, addr.to_string()))
                .collect();
            rollup.set_executor_address_list(executor_address_list);
            let rollup_clone = rollup.clone();
            rollup.update().map_err(Error::Database)?;

            let rollup_metadata = RollupMetadata::get(&rollup_info.id).map_err(Error::Database)?;

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

            let mut rollup_id_list =
                RollupIdList::get_mut_or(RollupIdList::default).map_err(Error::KvStoreError)?;
            rollup_id_list.insert(&rollup.rollup_id);
            rollup_id_list.update().map_err(Error::KvStoreError)?;

            let mut rollup_metadata = RollupMetadata::default();
            rollup_metadata.cluster_id = cluster_id.to_owned();

            rollup_metadata
                .put(&rollup.rollup_id)
                .map_err(Error::Database)?;

            Rollup::put(&rollup, &rollup.rollup_id).map_err(Error::Database)?;

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
