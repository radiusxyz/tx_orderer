use std::fmt::Display;

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PlatForm {
    Local,
    Ethereum,
}

impl Display for PlatForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatForm::Local => write!(f, "local"),
            PlatForm::Ethereum => write!(f, "ethereum"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SequencingFunctionType {
    Liveness,
    Validation,
}

impl Display for SequencingFunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SequencingFunctionType::Liveness => write!(f, "liveness"),
            SequencingFunctionType::Validation => write!(f, "validation"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    Local,
    Radius,
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::Local => write!(f, "local"),
            ServiceType::Radius => write!(f, "radius"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SequencingInfo {
    pub platform: PlatForm,
    pub sequencing_function_type: SequencingFunctionType,
    pub service_type: ServiceType,

    pub provider_rpc_url: IpAddress,
    pub provider_websocket_url: IpAddress,

    pub contract_address: Option<Address>,
}

impl SequencingInfo {
    pub fn new(
        platform: PlatForm,
        sequencing_function_type: SequencingFunctionType,
        service_type: ServiceType,
        provider_rpc_url: IpAddress,
        provider_websocket_url: IpAddress,
        contract_address: Option<Address>,
    ) -> Self {
        Self {
            platform,
            sequencing_function_type,
            service_type,
            provider_rpc_url,
            provider_websocket_url,
            contract_address,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SequencingInfoKey(PlatForm, SequencingFunctionType, ServiceType);

impl SequencingInfoKey {
    pub fn new(
        platform: PlatForm,
        sequencing_function_type: SequencingFunctionType,
        service_type: ServiceType,
    ) -> Self {
        Self(platform, sequencing_function_type, service_type)
    }

    pub fn platform(&self) -> &PlatForm {
        &self.0
    }

    pub fn sequencing_function_type(&self) -> &SequencingFunctionType {
        &self.1
    }

    pub fn service_type(&self) -> &ServiceType {
        &self.2
    }
}

impl Display for SequencingInfoKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.0, self.1, self.2)
    }
}
