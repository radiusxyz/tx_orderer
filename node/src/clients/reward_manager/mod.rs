mod errors;
mod types;

use std::sync::Arc;

pub use errors::*;
use radius_sdk::{
    json_rpc::client::{Id, RpcClient},
    signature::Address,
};
pub use types::*;

use crate::types::{ClusterId, RollupId};

pub struct RewardManagerClient {
    inner: Arc<RewardManagerClientInner>,
}

struct RewardManagerClientInner {
    rpc_url: String,
    rpc_client: Arc<RpcClient>,
}

impl Clone for RewardManagerClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl RewardManagerClient {
    pub fn new(rpc_url: impl AsRef<str>) -> Result<Self, RewardManagerError> {
        let inner = RewardManagerClientInner {
            rpc_url: rpc_url.as_ref().to_owned(),
            rpc_client: RpcClient::new().map_err(RewardManagerError::Initialize)?,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub async fn get_create_task_reward_data_list(
        &self,
        cluster_id: &ClusterId,
        rollup_id: &RollupId,
    ) -> Result<(u64, Vec<Address>, Vec<[u8; 32]>, Vec<u64>, Vec<u64>), RewardManagerError> {
        let params = GetCreateTaskRewards {
            rollup_id: rollup_id.to_owned(),
            cluster_id: cluster_id.to_owned(),
        };

        tracing::info!("Get rewards: {:?}", params);

        let get_create_task_rewards_response: GetCreateTaskRewardsResponse = self
            .inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetCreateTaskRewards::METHOD_NAME,
                &params,
                Id::Null,
            )
            .await
            .map_err(RewardManagerError::Register)?;

        if get_create_task_rewards_response
            .distribution_data_list
            .len()
            == 0
        {
            return Ok((
                get_create_task_rewards_response.pending_reward_task_index,
                vec![],
                vec![],
                vec![],
                vec![],
            ));
        }

        let vault_address_list: Vec<Address> = get_create_task_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.vault_address.clone())
            .collect();

        let operator_merkle_root_list: Vec<[u8; 32]> = get_create_task_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.operator_merkle_root.clone())
            .collect();

        let total_staker_reward_list: Vec<u64> = get_create_task_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.total_staker_reward.clone())
            .collect();

        let total_operator_reward_list: Vec<u64> = get_create_task_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.total_operator_reward.clone())
            .collect();

        Ok((
            get_create_task_rewards_response.pending_reward_task_index,
            vault_address_list,
            operator_merkle_root_list,
            total_staker_reward_list,
            total_operator_reward_list,
        ))
    }

    pub async fn get_respond_task_reward_data_list(
        &self,
        cluster_id: &ClusterId,
        rollup_id: &RollupId,
    ) -> Result<(u64, Vec<Address>, Vec<[u8; 32]>, Vec<u64>, Vec<u64>), RewardManagerError> {
        let params = GetRespondTaskRewards {
            rollup_id: rollup_id.to_owned(),
            cluster_id: cluster_id.to_owned(),
        };

        tracing::info!("Get rewards: {:?}", params);

        let get_create_task_rewards_response: GetRespondTaskRewardsResponse = self
            .inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetRespondTaskRewards::METHOD_NAME,
                &params,
                Id::Null,
            )
            .await
            .map_err(RewardManagerError::Register)?;

        if get_create_task_rewards_response
            .distribution_data_list
            .len()
            == 0
        {
            return Ok((
                get_create_task_rewards_response.pending_reward_task_index,
                vec![],
                vec![],
                vec![],
                vec![],
            ));
        }

        let vault_address_list: Vec<Address> = get_create_task_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.vault_address.clone())
            .collect();

        let operator_merkle_root_list: Vec<[u8; 32]> = get_create_task_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.operator_merkle_root.clone())
            .collect();

        let total_staker_reward_list: Vec<u64> = get_create_task_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.total_staker_reward.clone())
            .collect();

        let total_operator_reward_list: Vec<u64> = get_create_task_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.total_operator_reward.clone())
            .collect();

        Ok((
            get_create_task_rewards_response.pending_reward_task_index,
            vault_address_list,
            operator_merkle_root_list,
            total_staker_reward_list,
            total_operator_reward_list,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_rewards_success() {
        let reward_manager_client =
            RewardManagerClient::new("https://a0f9-59-10-110-198.ngrok-free.app/rewards").unwrap();

        let cluster_id = "radius";
        let rollup_id = "rollup_id_2";

        let (
            task_id,
            vault_address_list,
            operator_merkle_root_list,
            total_staker_reward_list,
            total_operator_reward_list,
        ) = reward_manager_client
            .get_create_task_reward_data_list(cluster_id, rollup_id)
            .await
            .unwrap();

        println!(
            "create task - task_id: {:?} / {:?} / {:?} / {:?} / {:?}",
            task_id,
            vault_address_list,
            operator_merkle_root_list,
            total_staker_reward_list,
            total_operator_reward_list
        );

        let (
            task_id,
            vault_address_list,
            operator_merkle_root_list,
            total_staker_reward_list,
            total_operator_reward_list,
        ) = reward_manager_client
            .get_respond_task_reward_data_list(cluster_id, rollup_id)
            .await
            .unwrap();

        println!(
            "respond task - task_id: {:?} / {:?} / {:?} / {:?} / {:?}",
            task_id,
            vault_address_list,
            operator_merkle_root_list,
            total_staker_reward_list,
            total_operator_reward_list
        );
    }
}
