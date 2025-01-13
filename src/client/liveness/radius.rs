use std::{collections::BTreeMap, str::FromStr, sync::Arc};

use radius_sdk::{
    liveness::radius::{
        publisher::Publisher,
        subscriber::Subscriber,
        types::{
            rpc::types::Header, Events, ILivenessRadius::RollupInfo, Liveness::LivenessEvents,
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
        .map_err(|error| Error::CreateLivenessClient(error.into()))?;

        let subscriber = Subscriber::new(
            liveness_info.liveness_websocket_url,
            liveness_info.contract_address,
        )
        .map_err(|error| Error::CreateLivenessClient(error.into()))?;

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
                    .initialize_event_handler(callback, liveness_client.clone())
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

async fn callback(events: Events, liveness_client: LivenessClient) {
    match events {
        Events::Block(block) => on_new_block(block, liveness_client).await,
        Events::LivenessEvents(liveness_event, log) => match liveness_event {
            LivenessEvents::RegisterSequencer(register_sequencer) => {
                let cluster_id = register_sequencer.clusterId;
                let platform_block_height = log.block_number.unwrap();
                let sequencer_index: usize = register_sequencer.index.try_into().unwrap();
                let sequencer_address = register_sequencer.sequencerAddress.to_string();
                let sequencer_rpc_info = liveness_client
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
                liveness_event_list.push((sequencer_index, sequencer_rpc_info));
                liveness_event_list.update().unwrap();
            }
            LivenessEvents::DeregisterSequencer(deregister_sequencer) => {
                let cluster_id = deregister_sequencer.clusterId;
                let platform_block_height = log.block_number.unwrap();
                let sequencer_address = deregister_sequencer.sequencerAddress.to_string();

                let mut liveness_event_list = LivenessEventList::get_mut_or(
                    &cluster_id,
                    platform_block_height,
                    LivenessEventList::default,
                )
                .unwrap();
                liveness_event_list.push(sequencer_address);
                liveness_event_list.update().unwrap();
            }
            _others => {}
        },
    }
}

async fn on_new_block(block: Header, liveness_client: LivenessClient) {
    let platform_block_height = block.number;
    let previous_block_height = platform_block_height.wrapping_sub(1);

    let cluster_id_list = ClusterIdList::get_or(
        liveness_client.platform(),
        liveness_client.service_provider(),
        ClusterIdList::default,
    )
    .unwrap();

    for cluster_id in cluster_id_list.iter() {
        if let Ok(cluster) =
            update_cluster(&liveness_client, cluster_id, previous_block_height).await
        {
            cluster
                .put(
                    liveness_client.platform(),
                    liveness_client.service_provider(),
                    cluster_id,
                    platform_block_height,
                )
                .unwrap();
        } else {
            initialize_new_cluster(&liveness_client, cluster_id, platform_block_height).await;
        }
    }
}

async fn update_cluster(
    liveness_client: &LivenessClient,
    cluster_id: &String,
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
            LivenessEventType::RegisterSequencer((sequencer_index, sequencer_rpc_info)) => {
                cluster.register_sequencer(*sequencer_index, sequencer_rpc_info.clone());
            }
            LivenessEventType::DeregisterSequencer(sequencer_address) => {
                cluster.deregister_sequencer(sequencer_address);
            }
        }
    }
    Ok(cluster)
}

async fn initialize_new_cluster(
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
) {
    let sequencer_rpc_infos =
        fetch_sequencer_rpc_infos(liveness_client, cluster_id, platform_block_height).await;
    let rollup_id_list =
        fetch_and_update_rollups(liveness_client, cluster_id, platform_block_height).await;

    let block_margin = liveness_client
        .publisher()
        .get_block_margin()
        .await
        .unwrap();

    let sequencer_address = address_from_str(
        liveness_client.platform(),
        liveness_client.publisher().address().to_string(),
    );

    let cluster = Cluster::new(
        sequencer_rpc_infos,
        rollup_id_list,
        sequencer_address,
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

async fn fetch_and_update_rollups(
    liveness_client: &LivenessClient,
    cluster_id: &str,
    platform_block_height: u64,
) -> Vec<String> {
    let rollup_info_list = liveness_client
        .publisher()
        .get_rollup_info_list(cluster_id, platform_block_height)
        .await
        .unwrap();

    for rollup_info in rollup_info_list.iter() {
        let validation_service_provider =
            ValidationServiceProvider::from_str(&rollup_info.validationInfo.serviceProvider)
                .unwrap();

        update_or_create_rollup(
            liveness_client.platform(),
            liveness_client.service_provider(),
            validation_service_provider,
            cluster_id,
            rollup_info,
        )
        .unwrap();
    }

    rollup_info_list
        .iter()
        .map(|info| info.rollupId.clone())
        .collect()
}

fn update_or_create_rollup(
    platform: Platform,
    service_provider: ServiceProvider,
    validation_service_provider: ValidationServiceProvider,
    cluster_id: &str,
    rollup_info: &RollupInfo,
) -> Result<(), Error> {
    match Rollup::get_mut(&rollup_info.rollupId) {
        Ok(mut rollup) => {
            let executor_addresses = rollup_info
                .executorAddresses
                .iter()
                .map(|addr| address_from_str(platform, addr.to_string()))
                .collect();
            rollup.set_executor_address_list(executor_addresses);
            rollup.update().map_err(Error::Database)?;
        }
        Err(_) => {
            let validation_info = RollupValidationInfo::new(
                platform,
                validation_service_provider,
                address_from_str(
                    platform,
                    rollup_info
                        .validationInfo
                        .validationServiceManager
                        .to_string(),
                ),
            );

            let executor_addresses: Vec<Address> = rollup_info
                .executorAddresses
                .iter()
                .map(|addr| address_from_str(platform, addr.to_string()))
                .collect();

            let rollup = Rollup::new(
                rollup_info.rollupId.clone(),
                RollupType::from_str(&rollup_info.rollupType).unwrap(),
                EncryptedTransactionType::Skde,
                address_from_str(platform, rollup_info.owner.to_string()),
                validation_info,
                OrderCommitmentType::from_str(&rollup_info.orderCommitmentType).unwrap(),
                executor_addresses,
                cluster_id.to_owned(),
                platform,
                service_provider,
            );

            Rollup::put(&rollup, &rollup.rollup_id).map_err(Error::Database)?;
        }
    }

    Ok(())
}

fn address_from_str(platform: Platform, address: String) -> Address {
    Address::from_str(platform.into(), &address).unwrap()
}
