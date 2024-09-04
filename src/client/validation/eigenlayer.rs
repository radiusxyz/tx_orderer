use std::sync::Arc;

use radius_sequencer_sdk::validation_eigenlayer::{
    publisher::Publisher, subscriber::Subscriber, types::Avs,
};
use tokio::time::{sleep, Duration};

use crate::error::Error;

pub struct ValidationClient {
    inner: Arc<ValidationClientInner>,
}

struct ValidationClientInner {
    publisher: Publisher,
    subscriber: Subscriber,
}

unsafe impl Send for ValidationClient {}

unsafe impl Sync for ValidationClient {}

impl Clone for ValidationClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl ValidationClient {
    pub fn new() -> Result<Self, Error> {
        todo!("Contract refactoring");
        // let publisher = Publisher::new();
    }

    pub fn publisher(&self) -> &Publisher {
        &self.inner.publisher
    }

    pub fn subscriber(&self) -> &Subscriber {
        &self.inner.subscriber
    }

    pub fn initialize_event_listener(&self) {
        let validation_client = self.clone();

        tokio::spawn(async move {
            loop {
                validation_client
                    .subscriber()
                    .initialize_event_handler(callback, validation_client.clone())
                    .await
                    .unwrap();

                tracing::warn!("Reconnecting the eigenlayer validation event listener..");
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
}

/// Todo: Need to change the contract code.
async fn callback(event: Avs::NewTaskCreated, context: ValidationClient) {
    // Todo: Handle block commitment events.
}
