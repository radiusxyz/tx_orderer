use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rollup {
    rollup_id: RollupId,
    rollup_type: RollupType,

    rollup_rpc_url: IpAddress,
    rollup_websocket_url: IpAddress,

    bundler_contract_address: Option<Address>,
}

impl Rollup {
    pub fn new(
        rollup_id: RollupId,
        rollup_type: RollupType,
        rollup_rpc_url: IpAddress,
        rollup_websocket_url: IpAddress,
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

    pub fn rollup_id(&self) -> &RollupId {
        &self.rollup_id
    }

    pub fn rollup_type(&self) -> &RollupType {
        &self.rollup_type
    }
}
