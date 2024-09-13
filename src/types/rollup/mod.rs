use std::str::FromStr;

use crate::{error::Error, types::prelude::*};

mod model;
mod rollup;
mod rollup_metadata;

pub use model::*;
pub use rollup::*;
pub use rollup_metadata::*;

pub type RollupIdList = Vec<String>;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RollupType {
    PolygonCdk,
}

impl From<RollupType> for radius_sequencer_sdk::signature::Platform {
    fn from(rollup_type: RollupType) -> Self {
        match rollup_type {
            RollupType::PolygonCdk => Self::Ethereum,
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
pub enum OrderCommitmentType {
    TxHash,
    OrderCommitment,
}

impl FromStr for OrderCommitmentType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tx_hash" | "TxHash" => Ok(Self::TxHash),
            "order_commitment" | "OrderCommitment" => Ok(Self::OrderCommitment),
            _ => Err(Error::NotSupportedRollupType),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
