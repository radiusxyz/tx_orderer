use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{ConfigOption, ConfigPath, CONFIG_FILE_NAME, DATABASE_DIR_NAME, SIGNING_KEY};
use crate::{
    error::Error,
    types::{Address, SigningKey},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    path: PathBuf,

    sequencer_rpc_url: String,
    internal_rpc_url: String,
    cluster_rpc_url: String,

    seeder_rpc_url: String,

    signing_key: SigningKey,

    is_using_zkp: bool,
}

impl Config {
    pub fn load(config_option: &mut ConfigOption) -> Result<Self, Error> {
        let config_path = match config_option.path.as_mut() {
            Some(config_path) => config_path.clone(),
            None => {
                let config_path: PathBuf = ConfigPath::default().as_ref().into();
                config_option.path = Some(config_path.clone());
                config_path
            }
        };

        // Read config file
        let config_file_path = config_path.join(CONFIG_FILE_NAME);
        let config_string =
            fs::read_to_string(&config_file_path).map_err(|_| Error::LoadConfigOption)?;

        // Parse String to TOML String
        let config_file: ConfigOption =
            toml::from_str(&config_string).map_err(|_| Error::ParseTomlString)?;

        // Merge configs from CLI input
        let merged_config_option = config_file.merge(config_option);

        // Read signing key
        let signing_key_path = config_path.join(SIGNING_KEY);
        let signing_key = SigningKey::from(fs::read_to_string(signing_key_path).unwrap());

        Ok(Config {
            path: config_path,
            sequencer_rpc_url: merged_config_option.sequencer_rpc_url.unwrap(),
            internal_rpc_url: merged_config_option.internal_rpc_url.unwrap(),
            cluster_rpc_url: merged_config_option.cluster_rpc_url.unwrap(),
            seeder_rpc_url: merged_config_option.seeder_rpc_url.unwrap(),
            signing_key,
            is_using_zkp: merged_config_option.is_using_zkp.unwrap(),
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn database_path(&self) -> PathBuf {
        self.path.join(DATABASE_DIR_NAME)
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    pub fn address(&self) -> Address {
        self.signing_key().get_address()
    }

    pub fn sequencer_rpc_url(&self) -> &String {
        &self.sequencer_rpc_url
    }

    pub fn internal_rpc_url(&self) -> &String {
        &self.internal_rpc_url
    }

    pub fn cluster_rpc_url(&self) -> &String {
        &self.cluster_rpc_url
    }

    pub fn seeder_rpc_url(&self) -> &String {
        &self.seeder_rpc_url
    }

    pub fn is_using_zkp(&self) -> bool {
        self.is_using_zkp
    }
}
