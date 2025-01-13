use std::str::FromStr;

use radius_sdk::signature::ChainType;

use crate::{error::Error, types::prelude::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Ethereum,
    Holesky,
    Local,
}

impl From<Platform> for ChainType {
    fn from(value: Platform) -> Self {
        match value {
            Platform::Ethereum => ChainType::Ethereum,
            Platform::Holesky => ChainType::Ethereum,
            Platform::Local => ChainType::Ethereum,
        }
    }
}

impl FromStr for Platform {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" => Ok(Self::Ethereum),
            "holesky" => Ok(Self::Holesky),
            "local" => Ok(Self::Local),
            _ => Err(Error::UnsupportedPlatform),
        }
    }
}
