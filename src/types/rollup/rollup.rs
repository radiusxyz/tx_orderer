use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rollup {
    rollup_id: String,
    rollup_type: RollupType,

    rollup_rpc_url: String,
    rollup_websocket_url: String,

    bundler_contract_address: Option<Address>,
}

impl Rollup {
    pub fn new(
        rollup_id: String,
        rollup_type: RollupType,
        rollup_rpc_url: String,
        rollup_websocket_url: String,
        bundler_contract_address: Option<Address>,
    ) -> Self {
        Self {
            rollup_id,
            rollup_type,
            rollup_rpc_url,
            rollup_websocket_url,
            bundler_contract_address,
        }
    }

    pub fn rollup_id(&self) -> &String {
        &self.rollup_id
    }

    pub fn rollup_type(&self) -> &RollupType {
        &self.rollup_type
    }
}
