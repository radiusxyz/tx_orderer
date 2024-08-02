use std::{collections::HashMap, sync::Arc};

use radius_sequencer_sdk::json_rpc::{Error as JsonRpcError, ErrorKind, RpcClient};
use serde_json::json;

use crate::{error::Error, types::*};

pub struct SeederClient(Arc<RpcClient>);

unsafe impl Send for SeederClient {}

unsafe impl Sync for SeederClient {}

impl Clone for SeederClient {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl SeederClient {
    pub fn new(seeder_rpc_url: impl AsRef<str>) -> Result<Self, Error> {
        let client = RpcClient::new(seeder_rpc_url).map_err(|error| {
            Error::RpcError(JsonRpcError::custom(ErrorKind::BuildClient, error))
        })?;

        Ok(Self(Arc::new(client)))
    }

    pub async fn register_sequencer_rpc_url(
        &self,
        sequencer_address: Address,
        sequencer_rpc_url: IpAddress,
    ) -> Result<(), Error> {
        let rpc_method = json!({
            "sequencer_address": sequencer_address,
            "sequencer_rpc_url": sequencer_rpc_url,
        });

        self.0
            .request("register_sequencer_rpc_url", rpc_method)
            .await
            .map_err(|_| Error::RegisterSequencer)
    }

    pub async fn get_sequencer_rpc_urls(
        &self,
        proposer_set_id: &ProposerSetId,
    ) -> Result<HashMap<Address, IpAddress>, Error> {
        let rpc_method = json! { proposer_set_id };

        self.0
            .request("get_sequencer_rpc_urls", rpc_method)
            .await
            .map_err(|_| Error::GetSequencerRpcUrlList)
    }
}
