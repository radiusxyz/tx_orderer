use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddSequencingInfo {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub payload: SequencingInfoPayload,
}

impl RpcParameter<AppState> for AddSequencingInfo {
    type Response = ();

    fn method() -> &'static str {
        "add_sequencing_info"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        tracing::info!(
            "Add sequencing info - platform: {:?}, service provider: {:?}, payload: {:?}",
            self.platform,
            self.service_provider,
            self.payload
        );

        // Save `LivenessClient` metadata.
        let mut sequencing_info_list = SequencingInfoList::get_mut_or(SequencingInfoList::default)?;
        sequencing_info_list.insert(self.platform, self.service_provider);
        sequencing_info_list.update()?;

        SequencingInfoPayload::put(&self.payload, self.platform, self.service_provider)?;

        match &self.payload {
            SequencingInfoPayload::Ethereum(payload) => {
                liveness::radius::LivenessClient::initialize(
                    context.clone(),
                    self.platform,
                    self.service_provider,
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
