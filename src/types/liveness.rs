use std::{
    collections::btree_set::{BTreeSet, Iter},
    str::FromStr,
};

use crate::{error::Error, types::prelude::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LivenessServiceProvider {
    Radius,
}

impl FromStr for LivenessServiceProvider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "radius" => Ok(Self::Radius),
            _ => Ok(Self::Radius),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, liveness_service_provider: LivenessServiceProvider))]
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
pub struct SequencingInfoList(BTreeSet<(Platform, LivenessServiceProvider)>);

impl SequencingInfoList {
    pub fn insert(
        &mut self,
        platform: Platform,
        liveness_service_provider: LivenessServiceProvider,
    ) {
        self.0.insert((platform, liveness_service_provider));
    }

    pub fn remove(
        &mut self,
        platform: Platform,
        liveness_service_provider: LivenessServiceProvider,
    ) {
        self.0.remove(&(platform, liveness_service_provider));
    }

    pub fn iter(&self) -> Iter<'_, (Platform, LivenessServiceProvider)> {
        self.0.iter()
    }
}
