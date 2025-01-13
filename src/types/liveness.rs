use std::{
    collections::btree_set::{BTreeSet, Iter},
    str::FromStr,
};

use crate::{error::Error, types::prelude::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceProvider {
    Radius,
}

impl FromStr for ValidationServiceProvider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "eigen_layer" | "EigenLayer" => Ok(Self::EigenLayer),
            "symbiotic" | "Symbiotic" => Ok(Self::Symbiotic),
            _ => Err(Error::UnsupportedValidationServiceProvider),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, service_provider: ServiceProvider))]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key())]
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
