mod rollup_metadata;

use std::str::FromStr;

use radius_sdk::{kvstore::Model, signature::ChainType};
pub use rollup_metadata::*;

use crate::{error::Error, types::prelude::*};

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
