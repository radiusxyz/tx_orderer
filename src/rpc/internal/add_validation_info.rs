use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddValidationInfo {
    pub platform: Platform,
    pub validation_service_provider: ValidationServiceProvider,
    pub payload: ValidationInfoPayload,
}

impl AddValidationInfo {
    pub const METHOD_NAME: &'static str = "add_validation_info";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        tracing::info!(
            "Add validation info - platform: {:?}, validation service provider: {:?}, payload: {:?}",
            parameter.platform,
            parameter.validation_service_provider,
            parameter.payload
        );

        // Save `ValidationClient` metadata.
        let mut validation_info_list = ValidationInfoList::get_mut_or(ValidationInfoList::default)?;
        validation_info_list.insert(parameter.platform, parameter.validation_service_provider);
        validation_info_list.update()?;

        ValidationInfoPayload::put(
            &parameter.payload,
            parameter.platform,
            parameter.validation_service_provider,
        )?;

        match &parameter.payload {
            ValidationInfoPayload::EigenLayer(payload) => {
                let signing_key = context.config().signing_key();

                let validation_client = validation::eigenlayer::ValidationClient::new(
                    parameter.platform,
                    parameter.validation_service_provider,
                    payload.clone(),
                    signing_key,
                )?;
                validation_client.initialize_event_listener();

                context
                    .add_validation_client(
                        parameter.platform,
                        parameter.validation_service_provider,
                        validation_client,
                    )
                    .await?;
            }
            ValidationInfoPayload::Symbiotic(payload) => {
                let signing_key = context.config().signing_key();

                let validation_client = validation::symbiotic::ValidationClient::new(
                    parameter.platform,
                    parameter.validation_service_provider,
                    payload.clone(),
                    signing_key,
                )?;
                validation_client.initialize_event_listener();

                context
                    .add_validation_client(
                        parameter.platform,
                        parameter.validation_service_provider,
                        validation_client,
                    )
                    .await?;
            }
        }

        Ok(())
    }
}
