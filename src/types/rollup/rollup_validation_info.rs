use radius_sdk::signature::Address;
use serde::{Deserialize, Serialize};

use super::{Platform, ValidationServiceProvider};
use crate::types::serialize_address;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RollupValidationInfo {
    pub platform: Platform,
    pub validation_service_provider: ValidationServiceProvider,

    #[serde(serialize_with = "serialize_address")]
    pub validation_service_manager: Address,
}

impl RollupValidationInfo {
    pub fn new(
        platform: Platform,
        validation_service_provider: ValidationServiceProvider,
        validation_service_manager: Address,
    ) -> Self {
        Self {
            platform,
            validation_service_provider,
            validation_service_manager,
        }
    }
}
