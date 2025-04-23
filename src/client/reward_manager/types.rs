use radius_sdk::signature::Address;
use serde::{Deserialize, Serialize};

use crate::types::{
    deserialize_hash, deserialize_u64_from_string, serialize_hash, ClusterId, RollupId,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCreateTaskRewards {
    pub cluster_id: ClusterId,
    pub rollup_id: RollupId,
}

impl GetCreateTaskRewards {
    pub const METHOD_NAME: &'static str = "get_create_task_rewards";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCreateTaskRewardsResponse {
    pub pending_reward_task_index: u64,

    #[serde(rename = "distribution_data")]
    pub distribution_data_list: Vec<RewardDistributionData>,
}

////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRespondTaskRewards {
    pub cluster_id: ClusterId,
    pub rollup_id: RollupId,
}

impl GetRespondTaskRewards {
    pub const METHOD_NAME: &'static str = "get_respond_task_rewards";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRespondTaskRewardsResponse {
    pub pending_reward_task_index: u64,

    #[serde(rename = "distribution_data")]
    pub distribution_data_list: Vec<RewardDistributionData>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RewardDistributionData {
    #[serde(rename = "vault")]
    pub vault_address: Address,

    #[serde(deserialize_with = "deserialize_u64_from_string")]
    pub total_staker_reward: u64,

    #[serde(deserialize_with = "deserialize_u64_from_string")]
    pub total_operator_reward: u64,

    #[serde(
        deserialize_with = "deserialize_hash",
        serialize_with = "serialize_hash"
    )]
    pub operator_merkle_root: [u8; 32],
}
