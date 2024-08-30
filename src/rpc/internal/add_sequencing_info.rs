use crate::{models::SequencingInfoModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddSequencingInfo {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub payload: Payload,
}

impl AddSequencingInfo {
    pub const METHOD_NAME: &'static str = "add_sequencing_info";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        // TODO (1): Check if the client exists. If the client exist, then add the cluster ID. Otherwise, initialize the liveness client.

        // TODO (2): Register on the seeder.

        Ok(())
    }
}
