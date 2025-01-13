use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddValidationInfo {
    pub platform: Platform,
    pub validation_service_provider: ValidationServiceProvider,
    pub validation_info: ValidationInfo,
}

impl AddValidationInfo {
    pub const METHOD_NAME: &'static str = "add_validation_info";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        tracing::info!(
            "Adding validation info - Platform: {:?}, Provider: {:?}, Info: {:?}",
            parameter.platform,
            parameter.validation_service_provider,
            parameter.validation_info
        );

        // Save `ValidationClient` metadata.
        let mut validation_service_providers =
            ValidationServiceProviders::get_mut_or(ValidationServiceProviders::default)?;
        validation_service_providers.insert(
            parameter.platform.clone(),
            parameter.validation_service_provider.clone(),
        );
        validation_service_providers.update()?;

        ValidationInfo::put(
            &parameter.validation_info,
            parameter.platform.clone(),
            parameter.validation_service_provider.clone(),
        )?;

        // Initialize the validation client
        Self::initialize_validation_client(
            context,
            parameter.platform,
            parameter.validation_service_provider,
            parameter.validation_info,
        )?;

        Ok(())
    }

    fn initialize_validation_client(
        context: Arc<AppState>,
        platform: Platform,
        provider: ValidationServiceProvider,
        validation_info: ValidationInfo,
    ) -> Result<(), RpcError> {
        match validation_info {
            ValidationInfo::EigenLayer(payload) => {
                validation::eigenlayer::ValidationClient::initialize(
                    (*context).clone(),
                    platform,
                    provider,
                    payload,
                );
            }
            ValidationInfo::Symbiotic(payload) => {
                validation::symbiotic::ValidationClient::initialize(
                    (*context).clone(),
                    platform,
                    provider,
                    payload,
                );
            }
        }
        Ok(())
    }
}
