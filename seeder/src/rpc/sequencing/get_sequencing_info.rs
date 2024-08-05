use sequencer::{
    models::{SequencingInfoKey, SequencingInfoModel},
    types::SequencingInfo,
};

use crate::{error::Error, rpc::prelude::*};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfo {
    pub sequencing_info_key: SequencingInfoKey,
}

// TODO:
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencingInfoResponse {
    sequencing_info: SequencingInfo,
}

impl GetSequencingInfo {
    pub const METHOD_NAME: &'static str = "get_sequencing_info";

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<()>,
    ) -> Result<GetSequencingInfoResponse, RpcError> {
        let parameter = parameter.parse::<GetSequencingInfo>()?;
        let sequencing_info_model = SequencingInfoModel::get()?;

        let sequencing_info = sequencing_info_model
            .sequencing_infos()
            .get(&parameter.sequencing_info_key)
            .ok_or(Error::GetSequencingInfo)?
            .clone();

        Ok(GetSequencingInfoResponse { sequencing_info })
    }
}
