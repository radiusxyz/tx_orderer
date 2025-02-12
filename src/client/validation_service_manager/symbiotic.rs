use std::sync::Arc;

use radius_sdk::validation::symbiotic::{
    publisher::Publisher, subscriber::Subscriber, types::ValidationServiceManager,
};
use tokio::time::{sleep, Duration};

use crate::{error::Error, state::AppState, types::*};

pub struct ValidationClient {
    inner: Arc<ValidationClientInner>,
}

struct ValidationClientInner {
    platform: Platform,
    validation_service_provider: ValidationServiceProvider,
    publisher: Publisher,
    subscriber: Subscriber,
}

impl Clone for ValidationClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl ValidationClient {
    pub fn platform(&self) -> Platform {
        self.inner.platform
    }

    pub fn validation_service_provider(&self) -> ValidationServiceProvider {
        self.inner.validation_service_provider
    }

    pub fn publisher(&self) -> &Publisher {
        &self.inner.publisher
    }

    pub fn subscriber(&self) -> &Subscriber {
        &self.inner.subscriber
    }

    pub fn new(
        platform: Platform,
        validation_service_provider: ValidationServiceProvider,
        symbiotic_validation_info: SymbioticValidationInfo,
        signing_key: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let publisher = Publisher::new(
            symbiotic_validation_info.validation_rpc_url,
            signing_key,
            symbiotic_validation_info
                .validation_contract_address
                .clone(),
        )
        .map_err(|error| Error::ValidationClient(error.into()))?;

        let subscriber = Subscriber::new(
            symbiotic_validation_info.validation_websocket_url,
            symbiotic_validation_info.validation_contract_address,
        )
        .map_err(|error| Error::ValidationClient(error.into()))?;

        let inner = ValidationClientInner {
            platform,
            validation_service_provider,
            publisher,
            subscriber,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn initialize(
        context: AppState,
        platform: Platform,
        validation_service_provider: ValidationServiceProvider,
        symbiotic_validation_info: SymbioticValidationInfo,
    ) {
        let handle = tokio::spawn({
            let context = context.clone();
            let validation_info = symbiotic_validation_info.clone();

            async move {
                let validation_client = Self::new(
                    platform,
                    validation_service_provider,
                    validation_info,
                    &context.config().signing_key,
                )
                .unwrap();

                context
                    .add_validation_client(
                        platform,
                        validation_service_provider,
                        validation_client.clone(),
                    )
                    .await
                    .unwrap();

                tracing::info!(
                    "Initializing Symbiotic validation event listener for {:?}, {:?}..",
                    platform,
                    validation_service_provider
                );
                validation_client
                    .subscriber()
                    .initialize_event_handler(callback, validation_client.clone())
                    .await
                    .unwrap();
            }
        });

        tokio::spawn(async move {
            if handle.await.is_err() {
                tracing::warn!("Reconnecting Symbiotic validation event listener..");
                sleep(Duration::from_secs(5)).await;
                Self::initialize(
                    context,
                    platform,
                    validation_service_provider,
                    symbiotic_validation_info,
                );
            }
        });
    }
}

async fn callback(event: ValidationServiceManager::NewTaskCreated, context: ValidationClient) {
    let rollup = Rollup::get(&event.rollupId).ok();
    if let Some(rollup) = rollup {
        let block = Block::get(&rollup.rollup_id, event.blockNumber.try_into().unwrap()).unwrap();

        tracing::info!("[Symbiotic] NewTaskCreated: clusterId: {:?} / rollupId: {:?} / referenceTaskIndex: {:?} / blockNumber: {:?} / blockCommitment: {:?}", event.clusterId, event.rollupId, event.referenceTaskIndex, event.blockNumber, event.blockCommitment);

        if block.block_creator_address != context.publisher().address() {
            for _ in 0..10 {
                match context
                    .publisher()
                    .respond_to_task(
                        &rollup.cluster_id,
                        &rollup.rollup_id,
                        event.referenceTaskIndex.try_into().unwrap(),
                        true,
                    )
                    .await
                    .map_err(|error| error.to_string())
                {
                    Ok(transaction_hash) => {
                        tracing::info!("[Symbiotic] respond_to_task: {:?}", transaction_hash);
                        break;
                    }
                    Err(error) => {
                        tracing::warn!("[Symbiotic] respond_to_task: {:?}", error);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }
}
