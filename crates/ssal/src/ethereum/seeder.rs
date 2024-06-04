use std::{str::FromStr, time::Duration};

use reqwest::{Client, Url};
use serde_json::json;

use crate::ethereum::{types::*, Error, ErrorKind};

pub struct Endpoint;

impl Endpoint {
    pub const REGISTER: &'static str = "/register";
    pub const DEREGISTER: &'static str = "/deregister";
    pub const ADDRESS_LIST: &'static str = "/address-list";
}

pub struct SeederClient {
    seeder_url: [Url; 3],
    client: Client,
}

unsafe impl Send for SeederClient {}

unsafe impl Sync for SeederClient {}

impl Clone for SeederClient {
    fn clone(&self) -> Self {
        Self {
            seeder_url: self.seeder_url.clone(),
            client: self.client.clone(),
        }
    }
}

impl SeederClient {
    pub fn new(seeder_address: impl AsRef<str>) -> Result<Self, Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|error| (ErrorKind::BuildSeederClient, error))?;
        let base_url = Url::from_str(seeder_address.as_ref())
            .map_err(|error| (ErrorKind::ParseSeederAddress, error))?;
        let seeder_url = [
            base_url
                .join(Endpoint::REGISTER)
                .map_err(|error| (ErrorKind::ParseSeederAddress, error))?,
            base_url
                .join(Endpoint::DEREGISTER)
                .map_err(|error| (ErrorKind::ParseSeederAddress, error))?,
            base_url
                .join(Endpoint::ADDRESS_LIST)
                .map_err(|error| (ErrorKind::ParseSeederAddress, error))?,
        ];

        Ok(Self { seeder_url, client })
    }

    pub async fn register(
        &self,
        signature: &Signature,
        public_key: &PublicKey,
    ) -> Result<(), Error> {
        let payload = json!({
            "signature": signature,
            "public_key": public_key,
        });
        self.client
            .post(self.seeder_url[0].clone())
            .json(&payload)
            .send()
            .await
            .map_err(|error| (ErrorKind::Register, error))?
            .error_for_status()
            .map_err(|error| (ErrorKind::Register, error))?;
        Ok(())
    }

    pub async fn deregister(
        &self,
        signature: &Signature,
        public_key: &PublicKey,
    ) -> Result<(), Error> {
        let payload = json!({
            "signature": signature,
            "public_key": public_key,
        });
        self.client
            .post(self.seeder_url[1].clone())
            .json(&payload)
            .send()
            .await
            .map_err(|error| (ErrorKind::Deregister, error))?
            .error_for_status()
            .map_err(|error| (ErrorKind::Deregister, error))?;
        Ok(())
    }

    pub async fn get_address_list(
        &self,
        sequencer_list: &Vec<PublicKey>,
    ) -> Result<Vec<Option<RpcAddress>>, Error> {
        let query = [("sequencer_list", sequencer_list)];
        let response: Vec<Option<RpcAddress>> = self
            .client
            .get(self.seeder_url[2].clone())
            .query(&query)
            .send()
            .await
            .map_err(|error| (ErrorKind::GetAddressList, error))?
            .error_for_status()
            .map_err(|error| (ErrorKind::GetAddressList, error))?
            .json()
            .await
            .map_err(|error| (ErrorKind::DeserializeResponse, error))?;
        Ok(response)
    }
}
