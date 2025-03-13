

use super::prelude::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Health;

impl RpcParameter<AppState> for Health {
    type Response = bool;

    fn method() -> &'static str {
        "health"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        Ok(true)
    }
}