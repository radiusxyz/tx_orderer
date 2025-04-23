use std::sync::Arc;

use radius_sdk::validation::symbiotic::{
    publisher::Publisher, subscriber::Subscriber, types::ValidationServiceManager,
};
use tokio::time::{sleep, Duration};

use crate::{client::reward_manager, error::Error, state::AppState, types::*};
const LOG_TARGET: &str = "client::validation_service_manager::symbiotic";

pub struct ValidationServiceManagerClient {
    inner: Arc<ValidationServiceManagerClientInner>,
}

struct ValidationServiceManagerClientInner {
    platform: Platform,
    validation_service_provider: ValidationServiceProvider,
    publisher: Publisher,
    subscriber: Subscriber,
}

impl Clone for ValidationServiceManagerClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl ValidationServiceManagerClient {
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
        .map_err(|error| Error::ValidationServiceManagerClient(error.into()))?;

        let subscriber = Subscriber::new(symbiotic_validation_info.validation_websocket_url)
            .map_err(|error| Error::ValidationServiceManagerClient(error.into()))?;

        let inner = ValidationServiceManagerClientInner {
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
                let validation_service_manager_client = Self::new(
                    platform,
                    validation_service_provider,
                    validation_info,
                    &context.config().signing_key,
                )
                .expect("Failed to initialize Symbiotic validation service manager client");

                context
                    .add_validation_service_manager_client(
                        platform,
                        validation_service_provider,
                        validation_service_manager_client.clone(),
                    )
                    .await
                    .expect("Failed to add Symbiotic validation service manager client");

                tracing::info!(
                    "Initializing Symbiotic validation event listener for {:?}, {:?}..",
                    platform,
                    validation_service_provider
                );

                let task_manager_contract_address = validation_service_manager_client
                    .publisher()
                    .get_task_manager_contract_address()
                    .await
                    .unwrap();

                validation_service_manager_client
                    .subscriber()
                    .initialize_task_manager_event_handler(
                        task_manager_contract_address,
                        callback,
                        (
                            context.reward_manager_client(),
                            validation_service_manager_client.clone(),
                        ),
                    )
                    .await
                    .expect("Failed to initialize Symbiotic validation event listener");
            }
        });

        tokio::spawn(async move {
            if handle.await.is_err() {
                tracing::warn!(
                    target: LOG_TARGET,
                    "Reconnecting Symbiotic validation event listener.."
                );
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

async fn callback(
    event: ValidationServiceManager::NewTaskCreated,
    (reward_manager_client, context): (
        &reward_manager::RewardManagerClient,
        ValidationServiceManagerClient,
    ),
) {
    let rollup = Rollup::get(&event.rollupId).ok();
    if let Some(rollup) = rollup {
        let batch = if let Ok(betch_number) = event.batchNumber.try_into() {
            // TODO: change
            match Batch::get(&rollup.rollup_id, betch_number) {
                Ok(batch) => batch,
                Err(err) => {
                    tracing::error!(
                        target: LOG_TARGET,
                        "Error getting batch: {} / batch_number: {:?}", err, betch_number
                    );
                    return;
                }
            }
        } else {
            tracing::error!(
                target: LOG_TARGET,
                "Error converting batch number");
            return;
        };

        tracing::info!(
            target: LOG_TARGET,
            "NewTaskCreated: clusterId: {:?} / rollupId: {:?} / referenceTaskIndex: {:?} / batchNumber: {:?} / batchCommitment: {:?}",
            event.clusterId,
            event.rollupId,
            event.referenceTaskIndex,
            event.batchNumber,
            event.batchCommitment
        );

        if batch.batch_creator_address != context.publisher().address() {
            let (
                reward_task_id,
                vault_address_list,
                operator_merkle_root_list,
                total_staker_reward_list,
                total_operator_reward_list,
            ) = reward_manager_client
                .get_respond_task_reward_data_list(&rollup.cluster_id, &rollup.rollup_id)
                .await
                .unwrap_or((0, vec![], vec![], vec![], vec![]));

            let reference_task_index = match event.referenceTaskIndex.try_into() {
                Ok(index) => index,
                Err(err) => {
                    tracing::error!(
                        target: LOG_TARGET,
                        "Error converting reference task index: {:?}",
                        err
                    );
                    return;
                }
            };

            if operator_merkle_root_list.len() != 0 {
                let (
                    check_vault_address_list,
                    check_operator_merkle_root_list,
                    check_total_staker_reward_list,
                    check_total_operator_reward_list,
                ) = match context
                    .publisher()
                    .get_distribution_data(&rollup.cluster_id, &rollup.rollup_id, reward_task_id)
                    .await
                {
                    Ok(data) => data,
                    Err(err) => {
                        tracing::error!(
                            target: LOG_TARGET,
                            "Error fetching distribution data: {:?}",
                            err
                        );
                        return;
                    }
                };

                if vault_address_list != check_vault_address_list
                    || operator_merkle_root_list != check_operator_merkle_root_list
                    || total_staker_reward_list != check_total_staker_reward_list
                    || total_operator_reward_list != check_total_operator_reward_list
                {
                    tracing::warn!(
                        target: LOG_TARGET,
                        "[Symbiotic] Distribution data mismatch.."
                    );
                    return;
                }
            }

            for _ in 0..10 {
                match context
                    .publisher()
                    .respond_to_task(
                        &rollup.cluster_id,
                        &rollup.rollup_id,
                        reference_task_index,
                        true,
                    )
                    .await
                    .map_err(|error| error.to_string())
                {
                    Ok(transaction_hash) => {
                        tracing::info!(
                            target: LOG_TARGET,
                            "respond_to_task: {:?}",
                            transaction_hash
                        );
                        break;
                    }
                    Err(error) => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            "respond_to_task: {:?}",
                            error
                        );
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }
}
