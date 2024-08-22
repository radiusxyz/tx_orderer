use sequencer::types::{Address, IpAddress};

use crate::{models::OperatorModel, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterRpcUrl {
    pub address: Address,
    pub rpc_url: IpAddress,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterRpcUrlResponse {
    pub success: bool,
}

impl RegisterRpcUrl {
    pub const METHOD_NAME: &'static str = "register_rpc_url";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<RegisterRpcUrlResponse, RpcError> {
        let parameter = parameter.parse::<RegisterRpcUrl>()?;

        // TODO: Remove this code
        // health_check(&parameter.rpc_url).await?;

        match OperatorModel::get(parameter.address.clone()) {
            // TODO: change(tmp logic when already registered)
            Ok(sequencer) => {
                tracing::warn!("Already registered sequencer: {:?}", sequencer);

                let sequencer =
                    OperatorModel::new(parameter.address.into(), parameter.rpc_url.into());

                sequencer.put()?;
            }
            Err(err) => {
                if err.is_none_type() {
                    let sequencer =
                        OperatorModel::new(parameter.address.into(), parameter.rpc_url.into());

                    sequencer.put()?;
                } else {
                    tracing::error!("Failed to add sequencer: {:?}", err);
                    return Err(err.into());
                }
            }
        };

        Ok(RegisterRpcUrlResponse { success: true })
    }
}
