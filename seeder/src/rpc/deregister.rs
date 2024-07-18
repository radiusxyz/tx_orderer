use super::prelude::*;

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<Deregister>()?;

    database()?.delete(&parameter.sequencer_address)?;

    tracing::info!("{:?}", parameter);

    Ok(())
}
