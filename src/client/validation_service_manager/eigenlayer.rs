use std::sync::Arc;

use radius_sdk::validation::eigenlayer::{
    publisher::Publisher,
    subscriber::Subscriber,
    types::{Avs, Bytes, IValidationServiceManager},
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
        eigen_layer_validation_info: EigenLayerValidationInfo,
        signing_key: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let publisher = Publisher::new(
            eigen_layer_validation_info.validation_rpc_url,
            signing_key,
            eigen_layer_validation_info.delegation_manager_contract_address,
            eigen_layer_validation_info.avs_directory_contract_address,
            eigen_layer_validation_info.stake_registry_contract_address,
            eigen_layer_validation_info.avs_contract_address.clone(),
        )
        .map_err(|error| Error::ValidationClient(error.into()))?;

        let subscriber = Subscriber::new(
            eigen_layer_validation_info.validation_websocket_url,
            eigen_layer_validation_info.avs_contract_address,
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
        eigen_layer_validation_info: EigenLayerValidationInfo,
    ) {
        let handle = tokio::spawn({
            let context = context.clone();
            let validation_info = eigen_layer_validation_info.clone();

            async move {
                let signing_key = &context.config().signing_key;
                let validation_client = Self::new(
                    platform,
                    validation_service_provider,
                    validation_info,
                    signing_key,
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
                    "Initializing EigenLayer validation event listener for {:?}, {:?}..",
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
                tracing::warn!("Reconnecting EigenLayer validation event listener..");
                sleep(Duration::from_secs(5)).await;
                Self::initialize(
                    context,
                    platform,
                    validation_service_provider,
                    eigen_layer_validation_info,
                );
            }
        });
    }
}

async fn callback(event: Avs::NewTaskCreated, context: ValidationClient) {
    let rollup = Rollup::get(&event.rollupId).ok();
    if let Some(rollup) = rollup {
        let block = Block::get(&rollup.rollup_id, event.task.blockNumber).unwrap();

        if block.block_creator_address != context.publisher().address() {
            let task = IValidationServiceManager::Task {
                commitment: Bytes::from_iter(&[0u8; 32]),
                blockNumber: 0,
                rollupId: rollup.rollup_id,
                clusterId: rollup.cluster_id,
                taskCreatedBlock: event.taskCreatedBlock,
            };

            let transaction_hash = context
                .publisher()
                .respond_to_task(task, event.taskIndex, Bytes::from_iter(&[0_u8; 64]))
                .await
                .unwrap();
            tracing::info!("[EigenLayer] respond_to_task: {:?}", transaction_hash);
        }
    }
}
