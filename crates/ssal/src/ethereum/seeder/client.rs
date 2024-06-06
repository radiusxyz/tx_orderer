use json_rpc::{RpcClient, RpcMethod};

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
        signature: Signature,
        public_key: PublicKey,
        sequencer_rpc_address: RpcAddress,
    ) -> Result<<Register as RpcMethod>::Response, Error> {
        let rpc_method = Register {
            signature,
            public_key,
            sequencer_rpc_address,
        };
        self.0
            .request(rpc_method)
            .await
            .map_err(|error| (ErrorKind::Register, error).into())
    }

    pub async fn deregister(
        &self,
        signature: Signature,
        public_key: PublicKey,
    ) -> Result<<Deregister as RpcMethod>::Response, Error> {
        let rpc_method = Deregister {
            signature,
            public_key,
        };
        self.0
            .request(rpc_method)
            .await
            .map_err(|error| (ErrorKind::Deregister, error).into())
    }

    pub async fn get_address_list(
        &self,
        sequencer_list: Vec<PublicKey>,
    ) -> Result<<GetAddressList as RpcMethod>::Response, Error> {
        let rpc_method = GetAddressList { sequencer_list };
        self.0
            .request(rpc_method)
            .await
            .map_err(|error| (ErrorKind::GetAddressList, error).into())
    }
}
