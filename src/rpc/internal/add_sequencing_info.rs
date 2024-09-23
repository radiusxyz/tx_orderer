use radius_sequencer_sdk::signature::PrivateKeySigner;

use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(try_from = "SequencingInfo")]
pub struct AddSequencingInfo {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub payload: SequencingInfoPayload,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// struct SequencingInfo {
//     platform: Platform,
//     service_provider: ServiceProvider,
//     payload: serde_json::Value,
// }

// impl TryFrom<SequencingInfo> for AddSequencingInfo {
//     type Error = Error;

//     fn try_from(value: SequencingInfo) -> Result<Self, Self::Error> {
//         match value.platform {
//             Platform::Ethereum => {
//                 let payload: LivenessRadius =
//
// serde_json::from_value(value.payload).map_err(Error::Deserialize)?;

//                 Ok(Self {
//                     platform: value.platform,
//                     service_provider: value.service_provider,
//                     payload: SequencingInfoPayload::Ethereum(payload),
//                 })
//             }
//             Platform::Local => {
//                 let payload: LivenessLocal =
//
// serde_json::from_value(value.payload).map_err(Error::Deserialize)?;

//                 Ok(Self {
//                     platform: value.platform,
//                     service_provider: value.service_provider,
//                     payload: SequencingInfoPayload::Local(payload),
//                 })
//             }
//         }
//     }
// }

impl AddSequencingInfo {
    pub const METHOD_NAME: &'static str = "add_sequencing_info";

    // pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) ->
    // Result<(), RpcError> {     let parameter = parameter.parse::<Self>()?;
    //     match &parameter.payload {
    //         SequencingInfoPayload::Ethereum(payload) => {
    //             let signing_key = context.config().signing_key();

    //             let mut sequencing_infos =
    // SequencingInfosModel::get_mut_or_default()?;

    //             sequencing_infos.insert(
    //                 parameter.platform,
    //                 parameter.service_provider,
    //                 parameter.payload.clone(),
    //             );
    //             sequencing_infos.update()?;

    //             liveness::radius::LivenessClient::new(
    //                 parameter.platform,
    //                 parameter.service_provider,
    //                 payload.clone(),
    //                 signing_key,
    //                 context.seeder_client().clone(),
    //             )?
    //             .initialize_event_listener();

    //             Ok(())
    //         }
    //         SequencingInfoPayload::Local(_payload) => {
    //             // liveness::local::LivenessClient::new()?;
    //             todo!("Implement 'LivenessClient' for local sequencing.");
    //         }
    //     }
    // }

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let parameter = parameter.parse::<Self>()?;

        // Save `LivenessClient` metadata.
        let mut sequencing_info_list = SequencingInfoListModel::get_mut_or_default()?;
        sequencing_info_list.insert(parameter.platform, parameter.service_provider);
        sequencing_info_list.update()?;

        SequencingInfoPayloadModel::put(
            parameter.platform,
            parameter.service_provider,
            &parameter.payload,
        )?;

        match &parameter.payload {
            SequencingInfoPayload::Ethereum(payload) => {
                let signing_key = context.config().signing_key();

                let signer = PrivateKeySigner::from_str(parameter.platform.into(), signing_key)?;
                context.add_signer(parameter.platform, signer).await?;

                let liveness_client = liveness::radius::LivenessClient::new(
                    parameter.platform,
                    parameter.service_provider,
                    payload.clone(),
                    signing_key,
                    context.seeder_client().clone(),
                )?;
                liveness_client.initialize_event_listener();

                context
                    .add_liveness_client(
                        parameter.platform,
                        parameter.service_provider,
                        liveness_client,
                    )
                    .await?;

                Ok(())
            }
            SequencingInfoPayload::Local(_payload) => {
                // liveness::local::LivenessClient::new()?;
                todo!("Implement 'LivenessClient' for local sequencing.");
            }
        }
    }
}
