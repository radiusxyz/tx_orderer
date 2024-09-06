use std::sync::Arc;

use radius_sequencer_sdk::liveness_radius::{
    publisher::Publisher, subscriber::Subscriber, types::Events,
};
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

async fn callback(events: Events, context: LivenessClient) {
    match events {
        Events::Block(block) => {
            let block_number = block.header.number.unwrap();

            // Get the cluster ID list for a given liveness client.
            let cluster_id_list =
                ClusterIdListModel::get_or_default(context.platform(), context.service_provider())
                    .unwrap();

            for cluster_id in cluster_id_list.iter() {
                let mut my_index: Option<usize> = None;

                // Get the sequencer address list for the cluster ID.
                let sequencer_address_list: Vec<String> = context
                    .publisher()
                    .get_sequencer_list(cluster_id, block_number)
                    .await
                    .unwrap()
                    .into_iter()
                    .enumerate()
                    .map(|(index, address)| {
                        if address == context.publisher().address() {
                            my_index = Some(index);
                        }

                        address.to_string()
                    })
                    .collect();

                // Get the rollup ID list from rollup information
                let rollup_id_list: Vec<String> = context
                    .publisher()
                    .get_rollup_info_list(cluster_id, block_number)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|rollup_info| rollup_info.rollupId)
                    .collect();

                let block_margin = context.publisher().get_block_margin().await.unwrap();

                let cluster_info = context
                    .seeder()
                    .get_cluster_info(sequencer_address_list.clone(), rollup_id_list.clone())
                    .await
                    .unwrap();

                for rollup_id in rollup_id_list {
                    let cluster_info = ClusterInfo::new(
                        cluster_info.sequencer_info.clone(),
                        my_index.unwrap(),
                        block_margin.try_into().unwrap(),
                    );
                    ClusterInfoModel::put(block_number, &rollup_id, &cluster_info).unwrap();
                }
            }
        }
        _others => {}
    }
}
