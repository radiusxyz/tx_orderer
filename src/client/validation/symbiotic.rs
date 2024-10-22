use std::sync::Arc;

use radius_sdk::validation_symbiotic::{
    publisher::Publisher,
    subscriber::Subscriber,
    types::{Bytes, ValidationServiceManager},
};
use tokio::time::{sleep, Duration};

use crate::{error::Error, types::*};

pub struct ValidationClient {
    inner: Arc<ValidationClientInner>,
}

struct ValidationClientInner {
    platform: Platform,
    service_provider: ServiceProvider,
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
    pub fn new(
        platform: Platform,
        service_provider: ServiceProvider,
        validation_info: ValidationSymbiotic,
        signing_key: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let publisher = Publisher::new(
            validation_info.validation_rpc_url,
            signing_key,
            validation_info.validation_contract_address.clone(),
        )
        .map_err(|error| Error::InitializeValidationClient(error.into()))?;

        let subscriber = Subscriber::new(
            validation_info.validation_websocket_url,
            validation_info.validation_contract_address,
        )
        .map_err(|error| Error::InitializeValidationClient(error.into()))?;

        let inner = ValidationClientInner {
            platform,
            service_provider,
            publisher,
            subscriber,
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
        let validation_client = self.clone();

        tokio::spawn(async move {
            loop {
                validation_client
                    .subscriber()
                    .initialize_event_handler(callback, validation_client.clone())
                    .await
                    .unwrap();

                tracing::warn!("Reconnecting Symbiotic validation event listener..");
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
}

async fn callback(event: ValidationServiceManager::NewTaskCreated, context: ValidationClient) {
    let rollup = Rollup::get(&event.rollupId).ok();
    if let Some(rollup) = rollup {
        let block = Block::get(rollup.rollup_id(), event.blockNumber.try_into().unwrap()).unwrap();

        if !block.is_leader {
            let transaction_hash = context
                .publisher()
                .respond_to_task(
                    rollup.cluster_id().to_owned(),
                    rollup.rollup_id().to_owned(),
                    event.referenceTaskIndex.try_into().unwrap(),
                    true,
                )
                .await
                .unwrap();
            tracing::info!("[Symbiotic] respond_to_task: {}", transaction_hash);
        }
    }
}
