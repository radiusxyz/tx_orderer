use std::str::FromStr;

use radius_sequencer_sdk::signature::ChainType;

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
