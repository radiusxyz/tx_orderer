use super::prelude::*;

pub async fn handler(
    parameter: RpcParameter,
    _context: Arc<()>,
) -> Result<Vec<Option<RpcAddress>>, RpcError> {
    let parameter = parameter.parse::<GetAddressList>()?;
    let sequencer_list: Vec<Option<RpcAddress>> = parameter
        .sequencer_list
        .iter()
        .map(|sequencer_public_key| {
            if let Some(database) = database().ok() {
                database
                    .get::<PublicKey, RpcAddress>(sequencer_public_key)
                    .ok()
            } else {
                None
            }
        })
        .collect();
    Ok(sequencer_list)
}
