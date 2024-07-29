use alloy::primitives::Address;

use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlList {
    pub proposer_set_id: String,
}

impl GetSequencerRpcUrlList {
    pub const METHOD_NAME: &'static str = stringify!(GetAddressList);
}

pub async fn handler(
    parameter: RpcParameter,
    _context: Arc<()>,
) -> Result<Vec<Option<String>>, RpcError> {
    let database = database()?;
    let parameter = parameter.parse::<GetSequencerRpcUrlList>()?;

    let sequencer_list: Vec<Address> = database.get(&parameter.proposer_set_id).unwrap();

    let sequencer_list: Vec<Option<String>> = sequencer_list
        .iter()
        .map(|sequencer_address| database.get::<Address, String>(sequencer_address).ok())
        .collect();

    Ok(sequencer_list)
}
