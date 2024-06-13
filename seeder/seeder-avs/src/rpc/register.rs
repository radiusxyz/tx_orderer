use super::prelude::*;

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<Register>()?;
    tracing::info!("{:?}", parameter);
    health_check(&parameter.sequencer_rpc_address).await?;
    database()
        .put(&parameter.public_key, &parameter.sequencer_rpc_address)
        .map_err(|error| error.into())
}
