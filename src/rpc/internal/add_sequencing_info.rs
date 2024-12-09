use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddSequencingInfo {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub payload: SequencingInfoPayload,
}

impl AddSequencingInfo {
    pub const METHOD_NAME: &'static str = "add_sequencing_info";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        tracing::info!(
            "Add sequencing info - platform: {:?}, service provider: {:?}, payload: {:?}",
            parameter.platform,
            parameter.service_provider,
            parameter.payload
        );

        // Save `LivenessClient` metadata.
        let mut sequencing_info_list = SequencingInfoList::get_mut_or(SequencingInfoList::default)?;
        sequencing_info_list.insert(parameter.platform, parameter.service_provider);
        sequencing_info_list.update()?;

        SequencingInfoPayload::put(
            &parameter.payload,
            parameter.platform,
            parameter.service_provider,
        )?;

        match &parameter.payload {
            SequencingInfoPayload::Ethereum(payload) => {
                liveness::radius::LivenessClient::initialize(
                    (*context).clone(),
                    parameter.platform,
                    parameter.service_provider,
                    payload.clone(),
                );
            }
            SequencingInfoPayload::Local(_payload) => {
                // liveness::local::LivenessClient::new()?;
                todo!("Implement 'LivenessClient' for local sequencing.");
            }
        }

        Ok(())
    }
}
