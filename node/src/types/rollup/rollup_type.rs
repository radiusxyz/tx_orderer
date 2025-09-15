use std::str::FromStr;

use radius_sdk::signature::ChainType;
use serde::{Deserialize, Serialize};

use crate::error::Error;

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
