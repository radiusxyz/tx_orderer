use crate::{rpc::prelude::*, task::create_batch};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBatchCreation {
    pub batch_creation_massage: BatchCreationMessage,
    pub leader_tx_orderer_signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchCreationMessage {
    pub rollup_id: String,
    pub batch_number: u64,
    pub batch_commitment: [u8; 32],
}

impl RpcParameter<AppState> for SyncBatchCreation {
    type Response = ();

    fn method() -> &'static str {
        "sync_batch_creation"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        // tracing::info!(
        //     "Change batch number - rollup id: {:?}, new batch number: {:?}",
        //     self.batch_change_massage.rollup_id,
        //     self.batch_change_massage.new_batch_number
        // );

        let rollup_id = self.batch_creation_massage.rollup_id;

        create_batch(
            context,
            &rollup_id,
            self.batch_creation_massage.batch_number,
            self.leader_tx_orderer_signature,
        );

        Ok(())
    }
}
