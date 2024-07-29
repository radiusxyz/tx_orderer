use json_rpc::ErrorKind;

use super::prelude::*;

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<Register>()?;

    health_check(&parameter.sequencer_rpc_url).await?;

    tracing::info!("{:?}", parameter);

    match database()?.get(&parameter.sequencer_address) {
        Ok(_) => {
            database()?.put(&parameter.sequencer_address, &parameter.sequencer_rpc_url)?;
            Ok(())
        }
        Err(_) => Err(ErrorKind::RegisterRpcMethod),
    }
}
