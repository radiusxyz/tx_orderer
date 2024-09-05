use crate::rpc::prelude::*;

/// 09/05
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "SequencingInfo")]
pub struct AddSequencingInfo {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub payload: SequencingInfoPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SequencingInfo {
    platform: Platform,
    service_provider: ServiceProvider,
    payload: serde_json::Value,
}

impl TryFrom<SequencingInfo> for AddSequencingInfo {
    type Error = Error;

    fn try_from(value: SequencingInfo) -> Result<Self, Self::Error> {
        match value.platform {
            Platform::Ethereum => {
                let payload: LivenessEthereum =
                    serde_json::from_value(value.payload).map_err(Error::Deserialize)?;

                Ok(Self {
                    platform: value.platform,
                    service_provider: value.service_provider,
                    payload: SequencingInfoPayload::Ethereum(payload),
                })
            }
            Platform::Local => {
                let payload: LivenessLocal =
                    serde_json::from_value(value.payload).map_err(Error::Deserialize)?;

                Ok(Self {
                    platform: value.platform,
                    service_provider: value.service_provider,
                    payload: SequencingInfoPayload::Local(payload),
                })
            }
        }
    }
}

impl AddSequencingInfo {
    pub const METHOD_NAME: &'static str = "add_sequencing_info";

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        match parameter.payload {
            SequencingInfoPayload::Ethereum(payload) => {
                // Todo: Fetching the signing key from the keystore.
                let signing_key = String::new();

                liveness::ethereum::LivenessClient::new(
                    parameter.platform,
                    parameter.service_provider,
                    payload,
                    signing_key,
                )?
                .initialize_event_listener();

                Ok(())
            }
            SequencingInfoPayload::Local(payload) => {
                // liveness::local::LivenessClient::new()?;
                todo!("Implement 'LivenessClient' for local sequencing.");
            }
        }
    }
}
