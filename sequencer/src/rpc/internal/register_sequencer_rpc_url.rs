use crate::{client::SeederClient, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterRpcUrl {
    rpc_url: IpAddress,
    address: Address,
}

impl RegisterRpcUrl {
    pub const METHOD_NAME: &'static str = stringify!(RegisterSequencerRpcUrl);

    pub async fn handler(parameter: RpcParameter, context: Arc<AppState>) -> Result<(), RpcError> {
        let seeder_rpc_url = context.config().seeder_rpc_url();
        let parameter = parameter.parse::<Self>()?;

        let seeder_client = SeederClient::new(seeder_rpc_url)?;

        seeder_client
            .register_rpc_url(parameter.address, parameter.rpc_url)
            .await
            .map_err(|error| error.into())
    }
}
