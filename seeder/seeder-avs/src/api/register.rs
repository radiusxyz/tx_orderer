use super::prelude::*;
use crate::util::health_check;

#[derive(Debug, Deserialize, Serialize)]
pub struct Register {
    signature: Signature,
    public_key: PublicKey,
}

impl Register {
    pub async fn handler(
        ConnectInfo(mut address): ConnectInfo<SocketAddr>,
        State(state): State<Database>,
        Json(payload): Json<Self>,
    ) -> Result<impl IntoResponse, Error> {
        let signer = payload
            .signature
            .recover(payload.public_key.as_bytes())
            .map_err(Error::boxed)?;
        if signer.0 == payload.public_key.0 {
            // Change the port to that of the default JSON RPC endpoint.
            address.set_port(8000);
            let sequencer_address = address.to_string();

            // Check if the sequencer is up and running.
            health_check(&sequencer_address)
                .await
                .map_err(Error::boxed)?;

            state
                .put(&payload.public_key, &sequencer_address)
                .map_err(Error::boxed)?;
            Ok((StatusCode::OK, Json(())))
        } else {
            Err(Error::SignatureMismatch)
        }
    }
}
