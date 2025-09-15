use std::{
    collections::btree_set::{BTreeSet, Iter},
    str::FromStr,
};

use crate::{error::Error, types::prelude::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationServiceProvider {
    EigenLayer,
    Symbiotic,
}

impl FromStr for ValidationServiceProvider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eigen_layer" | "eigenlayer" => Ok(Self::EigenLayer),
            "symbiotic" => Ok(Self::Symbiotic),
            _ => Ok(Self::Symbiotic),
        }
    }
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

impl ValidationInfo {
    pub fn platform(&self) -> &Platform {
        match self {
            ValidationInfo::EigenLayer(eigen_layer) => &eigen_layer.platform,
            ValidationInfo::Symbiotic(symbiotic) => &symbiotic.platform,
        }
    }

    pub fn validation_service_provider(&self) -> &ValidationServiceProvider {
        match self {
            ValidationInfo::EigenLayer(_) => &ValidationServiceProvider::EigenLayer,
            ValidationInfo::Symbiotic(_) => &ValidationServiceProvider::Symbiotic,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EigenLayerValidationInfo {
    platform: Platform,
    pub validation_rpc_url: String,
    pub validation_websocket_url: String,
    pub delegation_manager_contract_address: String,
    pub stake_registry_contract_address: String,
    pub avs_directory_contract_address: String,
    pub avs_contract_address: String,
}

impl EigenLayerValidationInfo {
    pub fn new(
        platform: Platform,
        validation_rpc_url: String,
        validation_websocket_url: String,
        delegation_manager_contract_address: String,
        stake_registry_contract_address: String,
        avs_directory_contract_address: String,
        avs_contract_address: String,
    ) -> Self {
        Self {
            platform,
            validation_rpc_url,
            validation_websocket_url,
            delegation_manager_contract_address,
            stake_registry_contract_address,
            avs_directory_contract_address,
            avs_contract_address,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SymbioticValidationInfo {
    platform: Platform,
    pub validation_rpc_url: String,
    pub validation_websocket_url: String,
    pub validation_contract_address: String,
}

impl SymbioticValidationInfo {
    pub fn new(
        platform: Platform,
        validation_rpc_url: String,
        validation_websocket_url: String,
        validation_contract_address: String,
    ) -> Self {
        Self {
            platform,
            validation_rpc_url,
            validation_websocket_url,
            validation_contract_address,
        }
    }
}
