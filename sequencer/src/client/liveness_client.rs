use std::sync::Arc;

use radius_sequencer_sdk::{
    json_rpc::{Error as JsonRpcError, ErrorKind, RpcClient},
    liveness::publisher::Publisher,
};
use serde_json::json;

use crate::{error::Error, types::*};

pub struct LivenessClient(Arc<Publisher>);

unsafe impl Send for LivenessClient {}

unsafe impl Sync for LivenessClient {}

impl Clone for LivenessClient {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl LivenessClient {
    pub fn new(
        liveness_provider_rpc_url: String,
        liveness_contract_address: String,
        signing_key: String,
    ) -> Result<Self, Error> {
        let liveness_client: Publisher = Publisher::new(
            liveness_provider_rpc_url,
            liveness_contract_address,
            signing_key,
        )?;

        Ok(Self(Arc::new(liveness_client)))
    }

    pub async fn get_block_height(&self) -> Result<BlockHeight, Error> {
        let block_height = self.0.get_block_number().await?;

        Ok(block_height)
    }

    pub async fn get_sequencer_address_list(
        &self,
        proposer_set_id: &ProposerSetId,
        liveness_block_height: Option<BlockHeight>,
    ) -> Result<Vec<Address>, Error> {
        let block_height = match liveness_block_height {
            Some(liveness_block_height) => liveness_block_height,
            None => self.get_block_height().await?,
        };

        let sequencer_address_list = self
            .0
            .get_sequencer_list(proposer_set_id, block_height)
            .await?
            .iter()
            .map(|address| Address::new(address.to_string()))
            .collect();

        Ok(sequencer_address_list)
    }

    pub async fn get_leader_sequencer_address(
        &self,
        proposer_set_id: &ProposerSetId,
        rollup_block_height: &BlockHeight,
        liveness_block_height: Option<BlockHeight>,
    ) -> Result<Address, Error> {
        let sequencer_address_list = self
            .get_sequencer_address_list(proposer_set_id, liveness_block_height)
            .await?;

        let leader_index = *rollup_block_height as usize % sequencer_address_list.len();

        Ok(sequencer_address_list[leader_index].clone())
    }
}
