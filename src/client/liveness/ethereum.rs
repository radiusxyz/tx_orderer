use std::sync::Arc;

use radius_sequencer_sdk::liveness_evm::{
    publisher::Publisher, subscriber::Subscriber, types::Events,
};
use tokio::time::{sleep, Duration};

use super::seeder::SeederClient;
use crate::{error::Error, models::*, types::*};

/// 09/05
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
        signing_key: impl AsRef<str>,
        ethereum_rpc_url: impl AsRef<str>,
        websocket_url: impl AsRef<str>,
        contract_address: impl AsRef<str>,
        seeder_rpc_url: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let publisher = Publisher::new(ethereum_rpc_url, signing_key, &contract_address)
            .map_err(|error| Error::InitializeLivenessClient(error.into()))?;
        let subscriber = Subscriber::new(websocket_url, contract_address)
            .map_err(|error| Error::InitializeLivenessClient(error.into()))?;

        let seeder = SeederClient::new(seeder_rpc_url)
            .map_err(|error| Error::InitializeLivenessClient(error.into()))?;

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
            // Get the cluster ID list for a given liveness client.
            let cluster_id_list =
                ClusterIdListModel::get_or_default(context.platform(), context.service_provider())
                    .unwrap();

            for cluster_id in cluster_id_list.iter() {
                let block_number = block.header.number.unwrap();

                // Get the sequencer address list given a cluster ID.
                let sequencer_address_list = context
                    .publisher()
                    .get_sequencer_list(cluster_id, block_number)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|address| address.to_string())
                    .collect();

                // Get [`ClusterInfo`] from the seeder.
                let sequencer_url_list: ClusterInfo = context
                    .seeder()
                    .get_cluster_info(
                        context.platform(),
                        context.service_provider(),
                        cluster_id.clone(),
                        sequencer_address_list,
                    )
                    .await
                    .unwrap();

                // Todo: Initialize validation client based on the rollup information fetched from the seeder.

                // Store the cluster information for the corresponding block number.
                ClusterInfoModel::put(
                    context.platform(),
                    context.service_provider(),
                    block_number,
                    &sequencer_url_list,
                )
                .unwrap();
            }
        }
        _others => {}
    }
}
