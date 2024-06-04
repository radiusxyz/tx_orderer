use super::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct AddressList {
    sequencer_list: Vec<PublicKey>,
}

impl AddressList {
    pub async fn handler(
        State(state): State<Database>,
        Query(parameter): Query<Self>,
    ) -> Result<impl IntoResponse, Error> {
        let address_list: Vec<Option<RpcAddress>> = parameter
            .sequencer_list
            .iter()
            .map(|sequencer_public_key| {
                state
                    .get::<PublicKey, RpcAddress>(&sequencer_public_key)
                    .ok()
            })
            .collect();
        Ok((StatusCode::OK, Json(address_list)))
    }
}
