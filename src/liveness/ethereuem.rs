use std::sync::Arc;

use radius_sequencer_sdk::liveness_evm::{
    publisher::Publisher, subscriber::Subscriber, types::Events,
};
use tokio::time::{sleep, Duration};

use crate::error::Error;

pub struct LivenessClient {
    inner: Arc<LivenessClientInner>,
}

struct LivenessClientInner {
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
        signing_key: impl AsRef<str>,
        rpc_url: impl AsRef<str>,
        websocket_url: impl AsRef<str>,
        contract_address: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let inner = LivenessClientInner {
            publisher: Publisher::new(rpc_url, signing_key, &contract_address)
                .map_err(|error| Error::InitializeLivenessClient(error.into()))?,
            subscriber: Subscriber::new(websocket_url, contract_address)
                .map_err(|error| Error::InitializeLivenessClient(error.into()))?,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
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
                    .initialize_event_handler(callback, ())
                    .await
                    .unwrap();

                tracing::warn!("Reconnecting the event handler..");
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
}

async fn callback(events: Events, context: ()) {
    match events {
        Events::Block(block) => {}
        // Skip handling other events.
        _others => {}
    }
}
