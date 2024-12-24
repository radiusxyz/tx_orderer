use std::str::FromStr;

use radius_sdk::signature::ChainType;

use super::prelude::*;
use crate::{client::liveness::seeder::SequencerRpcInfo, error::Error};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct RollupMetadata {
    pub rollup_block_height: u64,
    pub transaction_order: u64,
    pub merkle_tree: MerkleTree,

    pub cluster_id: String,

    pub platform_block_height: u64,
    pub is_leader: bool,
    pub leader_sequencer_rpc_info: SequencerRpcInfo,
}

impl RollupMetadata {
    pub fn new_merkle_tree(&mut self) {
        self.transaction_order = 0;
        self.merkle_tree = MerkleTree::new();
    }

    pub fn add_transaction_hash(&mut self, transaction_hash: &str) -> (u64, Vec<[u8; 32]>) {
        self.transaction_order += 1;
        self.merkle_tree.add_data(transaction_hash)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationInfo {
    pub platform: Platform,
    pub validation_service_provider: ValidationServiceProvider,

    #[serde(serialize_with = "serialize_address")]
    pub validation_service_manager: Address,
}

impl ValidationInfo {
    pub fn new(
        platform: Platform,
        validation_service_provider: ValidationServiceProvider,
        validation_service_manager: Address,
    ) -> Self {
        Self {
            platform,
            validation_service_provider,
            validation_service_manager,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct Rollup {
    pub rollup_id: String,
    pub rollup_type: RollupType,
    pub encrypted_transaction_type: EncryptedTransactionType,

    #[serde(serialize_with = "serialize_address")]
    pub owner: Address,
    pub validation_info: ValidationInfo,
    pub order_commitment_type: OrderCommitmentType,

    #[serde(serialize_with = "serialize_address_list")]
    pub executor_address_list: Vec<Address>,

    pub cluster_id: String,

    pub platform: Platform,
    pub service_provider: ServiceProvider,
}

impl Rollup {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rollup_id: String,
        rollup_type: RollupType,
        encrypted_transaction_type: EncryptedTransactionType,

        owner: Address,
        validation_info: ValidationInfo,
        order_commitment_type: OrderCommitmentType,
        executor_address_list: Vec<Address>,

        cluster_id: String,

        platform: Platform,
        service_provider: ServiceProvider,
    ) -> Self {
        Self {
            rollup_id,
            rollup_type,
            encrypted_transaction_type,
            owner,
            validation_info,
            order_commitment_type,
            executor_address_list,
            cluster_id,
            platform,
            service_provider,
        }
    }

    pub fn set_executor_address_list(&mut self, executor_address_list: Vec<Address>) {
        self.executor_address_list = executor_address_list;
    }
}

pub type RollupIdList = Vec<String>;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupType {
    PolygonCdk,
}

impl From<RollupType> for ChainType {
    fn from(value: RollupType) -> Self {
        match value {
            RollupType::PolygonCdk => ChainType::Ethereum,
        }
    }
}

impl FromStr for RollupType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "polygon_cdk" | "PolygonCdk" => Ok(Self::PolygonCdk),
            _ => Err(Error::UnsupportedRollupType),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptedTransactionType {
    Pvde,
    Skde,
    NotSupport,
}

impl Default for EncryptedTransactionType {
    fn default() -> Self {
        Self::NotSupport
    }
}

impl From<String> for EncryptedTransactionType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "pvde" | "Pvde" | "PVDE" => Self::Pvde,
            "skde" | "Skde" | "SKDE" => Self::Skde,
            _ => Self::NotSupport,
        }
    }
}
