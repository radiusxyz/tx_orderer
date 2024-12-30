use std::collections::btree_set::{BTreeSet, Iter};

use crate::types::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationServiceProvider {
    EigenLayer,
    Symbiotic,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Model)]
#[kvstore(key())]
pub struct ValidationServiceProviders(BTreeSet<(Platform, ValidationServiceProvider)>);

impl ValidationServiceProviders {
    pub fn insert(
        &mut self,
        platform: Platform,
        validation_service_provider: ValidationServiceProvider,
    ) {
        self.0.insert((platform, validation_service_provider));
    }

    pub fn remove(
        &mut self,
        platform: Platform,
        validation_service_provider: ValidationServiceProvider,
    ) {
        self.0.remove(&(platform, validation_service_provider));
    }

    pub fn iter(&self) -> Iter<'_, (Platform, ValidationServiceProvider)> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Model)]
#[kvstore(key(platform: Platform, validation_service_provider: ValidationServiceProvider))]
#[serde(untagged)]
pub enum ValidationInfo {
    EigenLayer(EigenLayerValidationInfo),
    Symbiotic(SymbioticValidationInfo),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EigenLayerValidationInfo {
    pub validation_rpc_url: String,
    pub validation_websocket_url: String,
    pub delegation_manager_contract_address: String,
    pub stake_registry_contract_address: String,
    pub avs_directory_contract_address: String,
    pub avs_contract_address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SymbioticValidationInfo {
    pub validation_rpc_url: String,
    pub validation_websocket_url: String,
    pub validation_contract_address: String,
}
