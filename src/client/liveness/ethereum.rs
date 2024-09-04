use std::sync::Arc;

use radius_sequencer_sdk::liveness_evm::{
    publisher::Publisher, subscriber::Subscriber, types::Events,
};
use tokio::time::{sleep, Duration};

use crate::{error::Error, models::*, types::*};

pub struct LivenessClient {
    inner: Arc<LivenessClientInner>,
}

struct LivenessClientInner {
    platform: Platform,
    service_provider: ServiceProvider,
    publisher: Publisher,
    subscriber: Subscriber,
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
        rpc_url: impl AsRef<str>,
        websocket_url: impl AsRef<str>,
        contract_address: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let inner = LivenessClientInner {
            platform,
            service_provider,
            publisher: Publisher::new(rpc_url, signing_key, &contract_address)
                .map_err(|error| Error::InitializeLivenessClient(error.into()))?,
            subscriber: Subscriber::new(websocket_url, contract_address)
                .map_err(|error| Error::InitializeLivenessClient(error.into()))?,
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

    pub fn initialize_event_listener(&self) {
        let liveness_client = self.clone();

        tokio::spawn(async move {
            loop {
                liveness_client
                    .subscriber()
                    .initialize_event_handler(callback, liveness_client.clone())
                    .await
                    .unwrap();

                tracing::warn!("Reconnecting the event handler..");
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
}

async fn callback(events: Events, context: LivenessClient) {
    match events {
        Events::Block(block) => {
            // Get the cluster ID list for a given liveness client.
            let cluster_id_list = ClusterIdListModel::get_or_default_mut(
                context.platform(),
                context.service_provider(),
            )
            .unwrap();

            for cluster_id in cluster_id_list.iter() {
                let block_number = block.header.number.unwrap();

                // Get the (sequencer, sequencer RPC URL) list.
                let sequencer_address_list = context
                    .publisher()
                    .get_sequencer_list(cluster_id, block_number)
                    .await
                    .unwrap();
            }
        }
        _others => {}
    }
}
