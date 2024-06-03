use super::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Deregister {
    signature: Signature,
    public_key: PublicKey,
}

impl Deregister {
    pub async fn handler(
        State(state): State<Database>,
        Json(payload): Json<Self>,
    ) -> Result<impl IntoResponse, Error> {
        let signer = payload
            .signature
            .recover(payload.public_key.as_bytes())
            .map_err(Error::boxed)?;
        if signer.0 == payload.public_key.0 {
            state.delete(&payload.public_key).map_err(Error::boxed)?;
            Ok((StatusCode::OK, Json(())))
        } else {
            Err(Error::SignatureMismatch)
        }
    }
}
