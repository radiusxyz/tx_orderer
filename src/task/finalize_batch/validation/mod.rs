use std::time::Duration;

use tokio::time::sleep;

use super::{BatchCommitment, Platform, Rollup, ValidationInfo, ValidationServiceProvider};
use crate::{client::validation_service_manager, state::AppState};

pub async fn submit_batch_commitment(
    context: AppState,
    rollup: &Rollup,
    validation_platform: Platform,
    validation_service_provider: ValidationServiceProvider,
    validation_info: ValidationInfo,
    batch_number: u64,
    batch_commitment: &BatchCommitment,
) {
    let batch_commitment = batch_commitment.as_bytes().unwrap();
    if (batch_number % 10) == 0 {
        tracing::info!(
            "Submit block commitment - rollup_id: {:?}, batch_number: {:?}, batch_commitment: {:?}",
            rollup.rollup_id,
            batch_number,
            batch_commitment
        );

        match validation_info {
            // TODO: we have to manage the nonce for the register block commitment.
            ValidationInfo::EigenLayer(_) => {
                let validation_service_manager_client: validation_service_manager::eigenlayer::ValidationServiceManagerClient =
                    context
                        .get_validation_service_manager_client(validation_platform, validation_service_provider)
                        .await
                        .unwrap();

                validation_service_manager_client
                    .publisher()
                    .register_block_commitment(
                        &rollup.cluster_id,
                        &rollup.rollup_id,
                        batch_number,
                        &batch_commitment,
                        // vault_addresses
                        // merkle_roots
                        // staker_rewards
                    )
                    .await
                    .unwrap();
            }
            ValidationInfo::Symbiotic(_) => {
                let (
                    reference_task_index,
                    vault_address_list,
                    operator_merkle_root_list,
                    total_staker_reward_list,
                    total_operator_reward_list,
                ) = context
                    .reward_manager_client()
                    .get_distribution_data_list(&rollup.cluster_id, &rollup.rollup_id)
                    .await
                    .unwrap_or((0, vec![], vec![], vec![], vec![]));

                let validation_service_manager_client: validation_service_manager::symbiotic::ValidationServiceManagerClient =
                    context
                        .get_validation_service_manager_client(validation_platform, validation_service_provider)
                        .await
                        .unwrap();

                let vault_address_list = vault_address_list
                    .iter()
                    .map(|address| address.as_hex_string())
                    .collect::<Vec<_>>();

                for _ in 0..10 {
                    match validation_service_manager_client
                        .publisher()
                        .register_block_commitment(
                            &rollup.cluster_id,
                            &rollup.rollup_id,
                            batch_number,
                            &batch_commitment,
                            reference_task_index,
                            vault_address_list.clone(),
                            operator_merkle_root_list.clone(),
                            total_staker_reward_list.clone(),
                            total_operator_reward_list.clone(),
                        )
                        .await
                        .map_err(|error| error.to_string())
                    {
                        Ok(transaction_hash) => {
                            tracing::info!(
                                "Registered block commitment - transaction hash: {:?}",
                                transaction_hash
                            );
                            break;
                        }
                        Err(error) => {
                            tracing::warn!("{:?}", error);
                            sleep(Duration::from_secs(2)).await;
                        }
                    }
                }
            }
        }
    }
}
