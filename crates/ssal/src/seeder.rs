use std::{str::FromStr, sync::Arc, time::Duration};

use primitives::{async_trait::async_trait, error::Error, serde_json::json};
use reqwest::{Client, ClientBuilder, StatusCode, Url};

use crate::types::*;

pub struct SeederClient {
    metadata: Arc<Metadata>,
    client: Client,
}

struct Metadata {
    seeder_url_list: Vec<Url>,
    signature: SequencerSignature,
    public_key: SequencerPublicKey,
}

unsafe impl Send for SeederClient {}

unsafe impl Sync for SeederClient {}

impl Clone for SeederClient {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            client: self.client.clone(),
        }
    }
}

impl SeederClient {
    pub fn new(
        seeder_address_list: &Vec<String>,
        signature: SequencerSignature,
        public_key: SequencerPublicKey,
    ) -> Result<Self, Error> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(Error::new)?;
        let seeder_url_list: Result<Vec<Url>, _> = seeder_address_list
            .iter()
            .map(|seeder_address| Url::from_str(seeder_address))
            .collect();
        let seeder_url_list = seeder_url_list.map_err(Error::new)?;

        let metadata = Metadata {
            seeder_url_list,
            signature,
            public_key,
        };

        Ok(Self {
            metadata: Arc::new(metadata),
            client,
        })
    }

    pub fn signature(&self) -> &SequencerSignature {
        &self.metadata.signature
    }

    pub fn public_key(&self) -> &SequencerPublicKey {
        &self.metadata.public_key
    }

    pub fn seeder_url_list(&self) -> &Vec<Url> {
        &self.metadata.seeder_url_list
    }
}

#[async_trait]
impl crate::SeederApi for SeederClient {
    async fn register(&self) -> Result<(), Error> {
        let payload = json!({
            "signature": self.signature(),
            "public_key": self.public_key(),
        });

        for seeder_url in self.seeder_url_list().iter() {
            let url = seeder_url.join("register").map_err(Error::new)?;
            match self.client.post(url).json(&payload).send().await {
                Ok(response) => match response.status() {
                    StatusCode::OK => return Ok(()),
                    _other_status_code => continue,
                },
                Err(_) => continue,
            }
        }

        Err(Error::from("All seeder nodes are unresponsive"))
    }

    async fn deregister(&self) -> Result<(), Error> {
        let payload = json!({
            "signature": self.signature(),
            "public_key": self.public_key(),
        });

        for seeder_url in self.seeder_url_list().iter() {
            let url = seeder_url.join("deregister").map_err(Error::new)?;
            match self.client.post(url).json(&payload).send().await {
                Ok(response) => match response.status() {
                    StatusCode::OK => return Ok(()),
                    _other_status_code => continue,
                },
                Err(_) => continue,
            }
        }

        Err(Error::from("All seeder nodes are unresponsive"))
    }

    async fn get_address_list(
        &self,
        sequencer_list: Vec<SequencerPublicKey>,
    ) -> Result<Vec<Option<SequencerAddress>>, Error> {
        let query = [("sequencer_list", sequencer_list)];

        for seeder_url in self.seeder_url_list().iter() {
            let url = seeder_url.join("/get-address-list").map_err(Error::new)?;
            match self.client.get(url).query(&query).send().await {
                Ok(response) => match response.json::<Vec<Option<SequencerAddress>>>().await {
                    Ok(sequencer_address_list) => return Ok(sequencer_address_list),
                    Err(_) => continue,
                },
                Err(_) => continue,
            }
        }

        Err(Error::from("All seeder nodes are unresponsive"))
    }
}
