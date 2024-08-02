use crate::{client::SeederClient, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSequencerRpcUrl {
    sequencer_rpc_url: IpAddress,
    sequencer_address: Address,
}

impl RegisterSequencerRpcUrl {
    pub const METHOD_NAME: &'static str = stringify!(RegisterSequencerRpcUrl);

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let seeder_rpc_url = context.config().seeder_rpc_url();
        let parameter = parameter.parse::<Self>()?;

        let seeder_client = SeederClient::new(seeder_rpc_url)?;

        seeder_client
            .register_sequencer_rpc_url(
                parameter.sequencer_address.into(),
                parameter.sequencer_rpc_url.into(),
            )
            .await
            .map_err(|error| error.into())
    }
}
