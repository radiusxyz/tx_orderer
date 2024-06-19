use std::sync::Arc;

use json_rpc::RpcClient;

use crate::avs::{seeder::rpc::*, types::*, Error, ErrorKind};

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
        let client = RpcClient::new(seeder_rpc_url, 5)
            .map_err(|error| (ErrorKind::BuildSeederClient, error))?;

        Ok(Self(Arc::new(client)))
    }

    pub async fn register(
        &self,
        sequencer_address: Address,
        sequencer_rpc_url: String,
    ) -> Result<(), Error> {
        let rpc_method = Register {
            sequencer_address,
            sequencer_rpc_url,
        };

        self.0
            .request(Register::METHOD_NAME, rpc_method)
            .await
            .map_err(|error| (ErrorKind::RegisterSequencer, error).into())
    }

    pub async fn deregister(&self, sequencer_address: Address) -> Result<(), Error> {
        let rpc_method = Deregister { sequencer_address };

        self.0
            .request(Deregister::METHOD_NAME, rpc_method)
            .await
            .map_err(|error| (ErrorKind::DeregisterSequencer, error).into())
    }

    pub async fn get_sequencer_rpc_urls(
        &self,
        sequencer_address_list: Vec<Address>,
    ) -> Result<Vec<Option<String>>, Error> {
        let rpc_method = GetSequencerRpcUrlList {
            sequencer_address_list,
        };

        self.0
            .request(GetSequencerRpcUrlList::METHOD_NAME, rpc_method)
            .await
            .map_err(|error| (ErrorKind::GetSequencerRpcUrl, error).into())
    }
}
