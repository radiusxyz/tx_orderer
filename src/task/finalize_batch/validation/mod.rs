use std::time::Duration;

use tokio::time::sleep;

use super::{Rollup, ValidationInfo, ValidationServiceProvider};
use crate::{client::validation_service_manager, state::AppState};

pub async fn submit_batch_commitment(
    context: AppState,
    rollup: &Rollup,
    batch_number: u64,
    batch_commitment: &[u8; 32],
) {
    // let validation_platform = context
    //     .get_validation_platform(rollup.cluster_id, rollup.rollup_id)
    //     .await
    //     .unwrap();

    // let validation_platform: Platform,
    // let validation_service_provider: ValidationServiceProvider,
    // let validation_info: ValidationInfo,

    tracing::info!(
        "Submit batch commitment - rollup_id: {:?}, batch_number: {:?},
    batch_commitment: {:?}",
        rollup.rollup_id,
        batch_number,
        batch_commitment
    );

    match rollup.validation_info {
        // TODO: we have to manage the nonce for the register batch commitment.
        ValidationInfo::EigenLayer(_) => {
            unimplemented!();
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
                .get_create_task_reward_data_list(&rollup.cluster_id, &rollup.rollup_id)
                .await
                .unwrap_or((0, vec![], vec![], vec![], vec![]));

            let vault_address_list = vault_address_list
                .iter()
                .map(|address| address.as_hex_string())
                .collect::<Vec<_>>();

            let validation_service_manager_client = match rollup.validation_info.validation_service_provider() {
                    ValidationServiceProvider::EigenLayer => {
                        panic!("EigenLayer validation service provider is not supported yet");
                    }
                    ValidationServiceProvider::Symbiotic => {
                        context
                    .get_validation_service_manager_client::<validation_service_manager::symbiotic::ValidationServiceManagerClient>(
                        rollup.validation_info.platform(),
                        &rollup.validation_info.validation_service_provider(),
                    )
                    .await
                    .unwrap()
                    }
                };

            for _ in 0..10 {
                match validation_service_manager_client
                    .publisher()
                    .register_batch_commitment(
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
                            "Registered batch commitment - transaction hash: {:?}",
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
