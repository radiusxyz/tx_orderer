use std::collections::btree_set::{BTreeSet, Iter};

use crate::types::prelude::*;

// TODO: Attributing Model
#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, validation_service_provider: ValidationServiceProvider))]
#[serde(untagged)]
pub enum ValidationInfoPayload {
    EigenLayer(ValidationEigenLayer),
    Symbiotic(ValidationSymbiotic),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationEigenLayer {
    pub validation_rpc_url: String,
    pub validation_websocket_url: String,
    pub delegation_manager_contract_address: String,
    pub stake_registry_contract_address: String,
    pub avs_directory_contract_address: String,
    pub avs_contract_address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidationSymbiotic {
    pub validation_rpc_url: String,
    pub validation_websocket_url: String,
    pub validation_contract_address: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key())]
pub struct ValidationInfoList(BTreeSet<(Platform, ValidationServiceProvider)>);

impl ValidationInfoList {
    pub fn insert(&mut self, platform: Platform, service_provider: ValidationServiceProvider) {
        self.0.insert((platform, service_provider));
    }

    pub fn remove(&mut self, platform: Platform, service_provider: ValidationServiceProvider) {
        self.0.remove(&(platform, service_provider));
    }

    pub fn iter(&self) -> Iter<'_, (Platform, ValidationServiceProvider)> {
        self.0.iter()
    }
}
