use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncMaxGasLimit {
    pub message: SyncMaxGasLimitMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncMaxGasLimitMessage {
    pub rollup_id: String,
    pub max_gas_limit: u64,
}

impl RpcParameter<AppState> for SyncMaxGasLimit {
    type Response = ();

    fn method() -> &'static str {
        "sync_max_gas_limit"
    }

    async fn handler(self, _context: AppState) -> Result<Self::Response, RpcError> {
        tracing::debug!(
            "Sync max gas limit - rollup id: {:?}, max gas limit: {:?}",
            self.message.rollup_id,
            self.message.max_gas_limit
        );

        let rollup_metadata = RollupMetadata::get(&self.message.rollup_id)?;
        let mut locked_rollup = Rollup::get_mut(&self.message.rollup_id)?;
        let cluster = Cluster::get(
            locked_rollup.platform,
            locked_rollup.service_provider,
            &locked_rollup.cluster_id,
            rollup_metadata.platform_block_height,
        )?;
        let sequencer_address_list = cluster.get_sequencer_address_list();

        let chain_type = locked_rollup.platform.into();
        for sequencer_address in sequencer_address_list {
            let verify_result =
                self.signature
                    .verify_message(chain_type, &self.message, sequencer_address);

            if verify_result.is_ok() {
                locked_rollup.max_gas_limit = self.message.max_gas_limit;
                locked_rollup.update()?;

                return Ok(());
            }
        }

        Err(Error::InvalidSignature)?
    }
}
