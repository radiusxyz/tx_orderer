use tokio::time::{sleep, Duration};

use tx_orderer_primitives::{Error, Platform, ValidationInfo, ValidationServiceProvider};
use crate::types::OrderCommitmentType;
use crate::{
    state::AppState,
    types::*,
};

pub struct ValidationService {
    app_state: AppState,
}

impl ValidationService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Main service loop - runs indefinitely processing validation events
    pub async fn run(&self) {
        tracing::info!("ValidationService started");
        
        loop {
            // Service runs in background, processing validation events
            // Implementation will be added when extracting logic from validation tasks
            sleep(Duration::from_secs(1)).await;
        }
    }

    /// Issue order commitment for a transaction
    pub async fn issue_order_commitment(
        &self,
        platform: Platform,
        rollup_id: RollupId,
        order_commitment_type: OrderCommitmentType,
        transaction_hash: RawTransactionHash,
        batch_number: u64,
        transaction_order: u64,
        pre_merkle_path: Vec<[u8; 32]>,
    ) -> Result<OrderCommitment, Error> {
        match order_commitment_type {
            OrderCommitmentType::TransactionHash => Ok(OrderCommitment::Single(
                SingleOrderCommitment::TransactionHash(TransactionHashOrderCommitment::new(
                    transaction_hash.as_string(),
                )),
            )),
            OrderCommitmentType::Sign => {
                let signer = self.app_state.get_signer(platform).await?;
                let order_commitment_data = OrderCommitmentData {
                    rollup_id,
                    batch_number,
                    transaction_hash: transaction_hash.as_string(),
                    transaction_order,
                    pre_merkle_path,
                };
                let order_commitment = SignOrderCommitment {
                    data: order_commitment_data.clone(),
                    signature: signer.sign_message(&order_commitment_data)?,
                };

                Ok(OrderCommitment::Single(SingleOrderCommitment::Sign(
                    order_commitment,
                )))
            }
        }
    }

    /// Submit batch commitment to validation contract
    pub async fn submit_batch_commitment(
        &self,
        rollup: &Rollup,
        batch_number: u64,
        batch_commitment: &[u8; 32],
    ) -> Result<(), Error> {
        tracing::info!(
            "Submit batch commitment - rollup_id: {:?}, batch_number: {:?}, batch_commitment: {:?}",
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
                ) = self.app_state
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
                            self.app_state
                        .get_validation_service_manager_client::<crate::clients::validation_service_manager::symbiotic::ValidationServiceManagerClient>(
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
                            batch_commitment,
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

        Ok(())
    }

    /// Check if current node is leader for the cluster
    pub async fn is_leader(
        &self,
        rollup: &Rollup,
    ) -> Result<bool, Error> {
        let cluster_metadata = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;

        Ok(cluster_metadata.is_leader)
    }

    /// Get leader external RPC URL for forwarding requests
    pub async fn get_leader_rpc_url(
        &self,
        rollup: &Rollup,
    ) -> Result<Option<String>, Error> {
        let cluster_metadata = ClusterMetadata::get(
            rollup.platform,
            rollup.liveness_service_provider,
            &rollup.cluster_id,
        )?;

        match cluster_metadata.leader_tx_orderer_rpc_info {
            Some(leader_info) => Ok(leader_info.external_rpc_url.clone()),
            None => Ok(None),
        }
    }
}