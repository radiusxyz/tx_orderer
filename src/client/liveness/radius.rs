use std::{str::FromStr, sync::Arc};

use radius_sdk::liveness_radius::{publisher::Publisher, subscriber::Subscriber, types::Events};
use tokio::time::{sleep, Duration};

use crate::{client::liveness::seeder::SeederClient, error::Error, types::*};

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

unsafe impl Send for LivenessClient {}

unsafe impl Sync for LivenessClient {}

impl Clone for LivenessClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
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

    pub fn initialize_event_listener(&self) {
        let liveness_client = self.clone();

        tokio::spawn(async move {
            loop {
                liveness_client
                    .subscriber()
                    .initialize_event_handler(callback, liveness_client.clone())
                    .await
                    .unwrap();

                tracing::warn!(
                    "Reconnecting the liveness event listener for {:?}, {:?}..",
                    liveness_client.platform(),
                    liveness_client.service_provider()
                );
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
}

async fn callback(events: Events, liveness_client: LivenessClient) {
    // TODO:
    match events {
        Events::Block(block) => {
            let platform_block_height = block.header.number.unwrap();

            // Get the cluster ID list for a given liveness client.
            let cluster_id_list = ClusterIdListModel::get_or_default(
                liveness_client.platform(),
                liveness_client.service_provider(),
            )
            .unwrap();

            let block_margin = liveness_client
                .publisher()
                .get_block_margin()
                .await
                .unwrap();

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
                    match RollupModel::get(&rollup_info.rollupId) {
                        Ok(_) => {}
                        Err(error) => {
                            if error.is_none_type() {
                                let order_commitment_type =
                                    OrderCommitmentType::from_str(&rollup_info.orderCommitmentType)
                                        .unwrap();
                                let rollup_type =
                                    RollupType::from_str(&rollup_info.rollupType).unwrap();
                                let validation_info = ValidationInfo::new(
                                    Platform::from_str(&rollup_info.validationInfo.platform)
                                        .unwrap(),
                                    ValidationServiceProvider::from_str(
                                        &rollup_info.validationInfo.serviceProvider,
                                    )
                                    .unwrap(),
                                );
                                let executor_address_list = rollup_info
                                    .executorAddresses
                                    .into_iter()
                                    .map(|address| address.to_string())
                                    .collect::<Vec<String>>();

                                let rollup = Rollup::new(
                                    rollup_info.rollupId.clone(),
                                    rollup_type,
                                    EncryptedTransactionType::Skde, // TODO
                                    rollup_info.owner.to_string(),
                                    validation_info,
                                    order_commitment_type,
                                    executor_address_list,
                                    cluster_id.to_owned(),
                                    liveness_client.platform(),
                                    liveness_client.service_provider(),
                                );

                                RollupModel::put(rollup.rollup_id(), &rollup).unwrap();

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

                ClusterModel::put(
                    liveness_client.platform(),
                    liveness_client.service_provider(),
                    &cluster_id,
                    platform_block_height,
                    &cluster,
                )
                .unwrap();
            }
        }
        _others => {}
    }
}
