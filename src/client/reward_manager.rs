use std::sync::Arc;

use radius_sdk::{
    json_rpc::client::{Id, RpcClient},
    signature::Address,
};
use serde::{Deserialize, Serialize};

use crate::types::{deserialize_hash, deserialize_u64_from_string, serialize_hash};

pub struct RewardManagerClient {
    inner: Arc<RewardManagerClientInner>,
}

struct RewardManagerClientInner {
    rpc_url: String,
    rpc_client: RpcClient,
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

    pub async fn distribution_data_list(
        &self,
        cluster_id: &str,
        rollup_id: &str,
    ) -> Result<(Vec<Address>, Vec<[u8; 32]>, Vec<u64>), RewardManagerError> {
        let params = GetRewards {
            rollup_id: rollup_id.to_owned(),
            cluster_id: cluster_id.to_owned(),
        };

        tracing::info!("Get rewards: {:?}", params);

        let get_rewards_response: GetRewardsResponse = self
            .inner
            .rpc_client
            .request(
                &self.inner.rpc_url,
                GetRewards::METHOD_NAME,
                &params,
                Id::Null,
            )
            .await
            .map_err(RewardManagerError::Register)?;

        let vault_address_list: Vec<Address> = get_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.vault_address.clone())
            .collect();

        let operator_merkle_root_list: Vec<[u8; 32]> = get_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.operator_merkle_root.clone())
            .collect();

        let total_staker_reward_list: Vec<u64> = get_rewards_response
            .distribution_data_list
            .iter()
            .map(|distribution_data| distribution_data.total_staker_reward.clone())
            .collect();

        Ok((
            vault_address_list,
            operator_merkle_root_list,
            total_staker_reward_list,
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRewards {
    pub cluster_id: String,
    pub rollup_id: String,
}

impl GetRewards {
    pub const METHOD_NAME: &'static str = "get_rewards";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRewardsResponse {
    pub task_id: u64,

    #[serde(rename = "distribution_data")]
    pub distribution_data_list: Vec<RewardDistributionData>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RewardDistributionData {
    #[serde(rename = "vault")]
    pub vault_address: Address,

    #[serde(deserialize_with = "deserialize_u64_from_string")]
    pub total_staker_reward: u64,

    #[serde(
        deserialize_with = "deserialize_hash",
        serialize_with = "serialize_hash"
    )]
    pub operator_merkle_root: [u8; 32],
}

#[derive(Debug)]
pub enum RewardManagerError {
    Initialize(radius_sdk::json_rpc::client::RpcClientError),
    Register(radius_sdk::json_rpc::client::RpcClientError),
}

impl std::fmt::Display for RewardManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RewardManagerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_rewards_success() {
        let reward_manager_client =
            RewardManagerClient::new("https://649a-59-10-110-198.ngrok-free.app/rewards").unwrap();

        let cluster_id = "radius";
        let rollup_id = "rollup_id_2";

        let (vault_address_list, operator_merkle_root_list, total_staker_reward_list) =
            reward_manager_client
                .distribution_data_list(cluster_id, rollup_id)
                .await
                .unwrap();

        println!(
            "{:?} / {:?} / {:?}",
            vault_address_list, operator_merkle_root_list, total_staker_reward_list
        );
    }
}
