use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetVersion {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetVersionResponse {
    pub version: Version,
}

impl RpcParameter<AppState> for GetVersion {
    type Response = GetVersionResponse;

    fn method() -> &'static str {
        "get_version"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        let version = Version::get()?;

        Ok(GetVersionResponse { version })
    }
}
