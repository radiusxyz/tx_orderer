use std::str::FromStr;

use radius_sdk::signature::ChainType;

use super::prelude::*;
use crate::error::Error;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct RollupMetadata {
    rollup_block_height: u64,
    transaction_order: u64,
    order_hash: OrderHash,

    is_leader: bool,
    platform_block_height: u64,

    cluster_id: String,
}

impl RollupMetadata {
    pub fn rollup_block_height(&self) -> u64 {
        self.rollup_block_height
    }

    pub fn transaction_order(&self) -> u64 {
        self.transaction_order
    }

    pub fn order_hash(&self) -> OrderHash {
        self.order_hash.clone()
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader
    }

    pub fn cluster_id(&self) -> &String {
        &self.cluster_id
    }

    pub fn platform_block_height(&self) -> u64 {
        self.platform_block_height
    }
}

impl RollupMetadata {
    pub fn set_is_leader(&mut self, is_leader: bool) {
        self.is_leader = is_leader;
    }

    pub fn set_cluster_id(&mut self, cluster_id: &String) {
        self.cluster_id.clone_from(cluster_id);
    }

    pub fn set_rollup_block_height(&mut self, block_height: u64) {
        self.rollup_block_height = block_height;
    }

    pub fn set_order_hash(&mut self, order_hash: OrderHash) {
        self.order_hash = order_hash;
    }

    pub fn set_transaction_order(&mut self, transaction_order: u64) {
        self.transaction_order = transaction_order;
    }

    pub fn set_platform_block_height(&mut self, platform_block_height: u64) {
        self.platform_block_height = platform_block_height;
    }

    pub fn increase_transaction_order(&mut self) -> u64 {
        self.transaction_order += 1;

        self.transaction_order
    }

    pub fn update_order_hash(&mut self, raw_transaction_hash: &RawTransactionHash) -> OrderHash {
        let previous_order_hash = self.order_hash.clone();
        self.order_hash = self.order_hash.update_order_hash(raw_transaction_hash);

        previous_order_hash
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationInfo {
    platform: Platform,
    service_provider: ValidationServiceProvider,
}

impl ValidationInfo {
    pub fn new(platform: Platform, service_provider: ValidationServiceProvider) -> Self {
        Self {
            platform,
            service_provider,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(rollup_id: &str))]
pub struct Rollup {
    rollup_id: String,
    rollup_type: RollupType,
    encrypted_transaction_type: EncryptedTransactionType,

    owner: String,
    validation_info: ValidationInfo,
    order_commitment_type: OrderCommitmentType,
    executor_address_list: Vec<String>,

    cluster_id: String,

    platform: Platform,
    service_provider: ServiceProvider,
}

impl Rollup {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rollup_id: String,
        rollup_type: RollupType,
        encrypted_transaction_type: EncryptedTransactionType,

        owner: String,
        validation_info: ValidationInfo,
        order_commitment_type: OrderCommitmentType,
        executor_address_list: Vec<String>,

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

    pub fn rollup_id(&self) -> &String {
        &self.rollup_id
    }

    pub fn rollup_type(&self) -> RollupType {
        self.rollup_type
    }

    pub fn encrypted_transaction_type(&self) -> EncryptedTransactionType {
        self.encrypted_transaction_type
    }

    pub fn order_commitment_type(&self) -> OrderCommitmentType {
        self.order_commitment_type
    }

    pub fn cluster_id(&self) -> &String {
        &self.cluster_id
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn service_provider(&self) -> ServiceProvider {
        self.service_provider
    }
}

pub type RollupIdList = Vec<String>;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupType {
    PolygonCdk,
}

impl Into<ChainType> for RollupType {
    fn into(self) -> ChainType {
        match self {
            Self::PolygonCdk => ChainType::Ethereum,
        }
    }
}

impl FromStr for RollupType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "polygon_cdk" | "PolygonCdk" => Ok(Self::PolygonCdk),
            _ => Err(Error::NotSupportedRollupType),
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
