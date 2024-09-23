use std::{
    collections::{
        // btree_map::{BTreeMap, Iter},
        btree_set::{BTreeSet, Iter},
    },
    str::FromStr,
};

use radius_sequencer_sdk::signature::ChainType;

use crate::{error::Error, types::prelude::*};

mod model;

pub use model::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Ethereum,
    Local,
}

impl Into<ChainType> for Platform {
    fn into(self) -> ChainType {
        match self {
            Self::Ethereum => ChainType::Ethereum,
            Self::Local => ChainType::Ethereum,
        }
    }
}

impl FromStr for Platform {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ethereum" | "Ethereum" => Ok(Self::Ethereum),
            "local" | "Local" => Ok(Self::Local),
            _ => Err(Error::NotSupportedPlatform),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceProvider {
    Radius,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationServiceProvider {
    EigenLayer,
    Symbiotic,
}

impl FromStr for ValidationServiceProvider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "eigen_layer" | "EigenLayer" => Ok(Self::EigenLayer),
            "symbiotic" | "Symbiotic" => Ok(Self::Symbiotic),
            _ => Err(Error::NotSupportedValidationServiceProvider),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(untagged)] - Deseiralize error: DeserializeAnyNotSupported
#[serde(untagged)]
pub enum SequencingInfoPayload {
    Ethereum(LivenessRadius),
    Local(LivenessLocal),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessRadius {
    pub liveness_rpc_url: String,
    pub liveness_websocket_url: String,
    pub contract_address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessLocal;

// #[derive(Clone, Debug, Default, Deserialize, Serialize)]
// pub struct SequencingInfos(BTreeMap<(Platform, ServiceProvider),
// SequencingInfoPayload>);

// impl SequencingInfos {
//     pub fn insert(
//         &mut self,
//         platform: Platform,
//         service_provider: ServiceProvider,
//         sequencing_info: SequencingInfoPayload,
//     ) {
//         self.0.insert((platform, service_provider), sequencing_info);
//     }

//     pub fn sequencing_infos(
//         &self,
//     ) -> &BTreeMap<(Platform, ServiceProvider), SequencingInfoPayload> {
//         &self.0
//     }

//     pub fn remove(&mut self, platform: Platform, service_provider:
// ServiceProvider) {         self.0.remove(&(platform, service_provider));
//     }

//     pub fn iter(&self) -> Iter<'_, (Platform, ServiceProvider),
// SequencingInfoPayload> {         self.0.iter()
//     }
// }

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SequencingInfoList(BTreeSet<(Platform, ServiceProvider)>);

impl SequencingInfoList {
    pub fn insert(&mut self, platform: Platform, service_provider: ServiceProvider) {
        self.0.insert((platform, service_provider));
    }

    pub fn remove(&mut self, platform: Platform, service_provider: ServiceProvider) {
        self.0.remove(&(platform, service_provider));
    }

    pub fn iter(&self) -> Iter<'_, (Platform, ServiceProvider)> {
        self.0.iter()
    }
}
