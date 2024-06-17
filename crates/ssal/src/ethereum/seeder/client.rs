use json_rpc::RpcClient;

use crate::ethereum::{seeder::rpc::*, types::*, Error, ErrorKind};

pub struct SeederClient(RpcClient);

impl SeederClient {
    pub fn new(seeder_rpc_address: impl AsRef<str>) -> Result<Self, Error> {
        let client = RpcClient::new(seeder_rpc_address, 5)
            .map_err(|error| (ErrorKind::BuildSeederClient, error))?;

        Ok(Self(client))
    }

    pub async fn register(
        &self,
        public_key: H160,
        sequencer_rpc_address: String,
    ) -> Result<(), Error> {
        let rpc_method = Register {
            public_key,
            sequencer_rpc_address,
        };

        self.0
            .request(Register::METHOD_NAME, rpc_method)
            .await
            .map_err(|error| (ErrorKind::RegisterSequencer, error).into())
    }

    pub async fn deregister(&self, public_key: H160) -> Result<(), Error> {
        let rpc_method = Deregister { public_key };

        self.0
            .request(Deregister::METHOD_NAME, rpc_method)
            .await
            .map_err(|error| (ErrorKind::DeregisterSequencer, error).into())
    }

    pub async fn get_address_list(
        &self,
        sequencer_list: Vec<H160>,
    ) -> Result<Vec<Option<String>>, Error> {
        let rpc_method = GetAddressList { sequencer_list };

        self.0
            .request(GetAddressList::METHOD_NAME, rpc_method)
            .await
            .map_err(|error| (ErrorKind::GetAddressList, error).into())
    }
}
