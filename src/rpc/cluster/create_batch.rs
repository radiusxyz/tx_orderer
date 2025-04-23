use crate::{rpc::prelude::*, task::create_batch};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBatchCreation {
    pub batch_creation_massage: BatchCreationMessage,
    pub leader_tx_orderer_signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchCreationMessage {
    pub rollup_id: RollupId,
    pub batch_number: u64,
    pub batch_commitment: [u8; 32],
    pub batch_creator_signature: Signature,
}

impl RpcParameter<AppState> for SyncBatchCreation {
    type Response = ();

    fn method() -> &'static str {
        "sync_batch_creation"
    }

    async fn handler(self, context: AppState) -> Result<Self::Response, RpcError> {
        let rollup_id = self.batch_creation_massage.rollup_id;

        create_batch(
            context,
            &rollup_id,
            self.batch_creation_massage.batch_number,
            self.batch_creation_massage.batch_creator_signature,
            self.leader_tx_orderer_signature,
        );

        Ok(())
    }
}
