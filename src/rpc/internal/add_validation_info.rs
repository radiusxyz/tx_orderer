use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddValidationInfo {
    pub platform: Platform,
    pub validation_service_provider: ValidationServiceProvider,
    pub validation_info: ValidationInfo,
}

impl RpcParameter<AppState> for AddValidationInfo {
    type Response = ();

    fn method() -> &'static str {
        "add_validation_info"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        tracing::info!(
            "Adding validation info - Platform: {:?}, Provider: {:?}, Info: {:?}",
            self.platform,
            self.validation_service_provider,
            self.validation_info
        );

        // Save `ValidationServiceManagerClient` metadata.
        let mut validation_service_providers =
            ValidationServiceProviders::get_mut_or(ValidationServiceProviders::default)?;
        validation_service_providers.insert(
            self.platform.clone(),
            self.validation_service_provider.clone(),
        );
        validation_service_providers.update()?;

        ValidationInfo::put(
            &self.validation_info,
            self.platform.clone(),
            self.validation_service_provider.clone(),
        )?;

        // Initialize the validation client
        Self::initialize_validation_service_manager_client(
            context,
            self.platform,
            self.validation_service_provider,
            self.validation_info,
        )?;

        Ok(())
    }
}

impl AddValidationInfo {
    fn initialize_validation_service_manager_client(
        context: AppState,
        platform: Platform,
        provider: ValidationServiceProvider,
        validation_info: ValidationInfo,
    ) -> Result<(), RpcError> {
        match validation_info {
            ValidationInfo::EigenLayer(payload) => {
                validation_service_manager::eigenlayer::ValidationServiceManagerClient::initialize(
                    context.clone(),
                    platform,
                    provider,
                    payload,
                );
            }
            ValidationInfo::Symbiotic(payload) => {
                validation_service_manager::symbiotic::ValidationServiceManagerClient::initialize(
                    context.clone(),
                    platform,
                    provider,
                    payload,
                );
            }
        }
        Ok(())
    }
}
