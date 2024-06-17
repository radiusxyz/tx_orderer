use super::prelude::*;

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<Deregister>()?;
    tracing::info!("{:?}", parameter);

    database()?
        .delete(&parameter.public_key)
        .map_err(|error| error.into())
}
