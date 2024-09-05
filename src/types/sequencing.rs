use std::collections::btree_map::{BTreeMap, Iter};

use crate::types::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Ethereum,
    Local,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceProvider {
    Radius,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SequencingInfoPayload {
    Ethereum(LivenessEthereum),
    Local(LivenessLocal),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessEthereum {
    pub liveness_rpc_url: String,
    pub liveness_websocket_url: String,
    pub contract_address: String,
    pub seeder_rpc_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LivenessLocal {
    pub seeder_rpc_url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SequencingInfoList(BTreeMap<(Platform, ServiceProvider), SequencingInfoPayload>);

impl SequencingInfoList {
    pub fn insert(
        &mut self,
        platform: Platform,
        service_provider: ServiceProvider,
        sequencing_info: SequencingInfoPayload,
    ) {
        self.0.insert((platform, service_provider), sequencing_info);
    }

    pub fn remove(&mut self, platform: Platform, service_provider: ServiceProvider) {
        self.0.remove(&(platform, service_provider));
    }

    pub fn iter(&self) -> Iter<'_, (Platform, ServiceProvider), SequencingInfoPayload> {
        self.0.iter()
    }
}
