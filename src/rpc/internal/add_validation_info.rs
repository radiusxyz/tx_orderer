use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddValidationInfo {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub payload: ValidationInfoPayload,
}

impl AddValidationInfo {
    pub const METHOD_NAME: &'static str = "add_validation_info";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        // Save `ValidationClient` metadata.
        let mut sequencing_info_list = ValidationInfoListModel::get_mut_or_default()?;
        sequencing_info_list.insert(parameter.platform, parameter.service_provider);
        sequencing_info_list.update()?;

        ValidationInfoPayloadModel::put(
            parameter.platform,
            parameter.service_provider,
            &parameter.payload,
        )?;

        match &parameter.payload {
            ValidationInfoPayload::EigenLayer(payload) => {
                let signing_key = context.config().signing_key();

                let validation_client = validation::eigenlayer::ValidationClient::new(
                    parameter.platform,
                    parameter.service_provider,
                    payload.clone(),
                    signing_key,
                )?;
                validation_client.initialize_event_listener();

                context
                    .add_validation_client(
                        parameter.platform,
                        parameter.service_provider,
                        validation_client,
                    )
                    .await?;
            }
            ValidationInfoPayload::Symbiotic(_) => {
                todo!("Implement 'LivenessClient' for local sequencing.");
            }
        }

        Ok(())
    }
}
