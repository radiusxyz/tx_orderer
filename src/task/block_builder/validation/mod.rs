use std::time::Duration;

use tokio::time::sleep;

use super::{BlockCommitment, Platform, Rollup, ValidationInfo, ValidationServiceProvider};
use crate::{client::validation_service_manager, state::AppState};

pub async fn submit_block_commitment(
    context: AppState,
    rollup: &Rollup,
    validation_platform: Platform,
    validation_service_provider: ValidationServiceProvider,
    validation_info: ValidationInfo,
    rollup_block_height: u64,
    block_commitment: &BlockCommitment,
) {
    let block_commitment = block_commitment.as_bytes().unwrap();
    if (rollup_block_height % 201600) == 0 {
        tracing::info!(
            "Submit block commitment - rollup_id: {:?}, rollup_block_height: {:?}, block_commitment: {:?}",
            rollup.rollup_id,
            rollup_block_height,
            block_commitment
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
                        rollup_block_height,
                        &block_commitment,
                    )
                    .await
                    .unwrap();
            }
            ValidationInfo::Symbiotic(_) => {
                let validation_service_manager_client: validation_service_manager::symbiotic::ValidationServiceManagerClient =
                    context
                        .get_validation_service_manager_client(validation_platform, validation_service_provider)
                        .await
                        .unwrap();

                for _ in 0..10 {
                    match validation_service_manager_client
                        .publisher()
                        .register_block_commitment(
                            &rollup.cluster_id,
                            &rollup.rollup_id,
                            rollup_block_height,
                            &block_commitment,
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
