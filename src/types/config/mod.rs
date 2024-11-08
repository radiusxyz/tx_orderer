mod config_option;
mod config_path;
mod config_register_validator;

use std::{fs, path::PathBuf};

pub use config_option::*;
pub use config_path::*;
pub use config_register_validator::*;
use serde::{Deserialize, Serialize};

pub const DEFAULT_HOME_PATH: &str = ".radius";
pub const DATABASE_DIR_NAME: &str = "database";
pub const CONFIG_FILE_NAME: &str = "Config.toml";
pub const SIGNING_KEY_PATH: &str = "signing_key";
pub const DEFAULT_SIGNING_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    path: PathBuf,

    sequencer_rpc_url: String,
    internal_rpc_url: String,
    cluster_rpc_url: String,

    seeder_rpc_url: String,

    distributed_key_generation_rpc_url: String,

    signing_key: String,

    is_using_zkp: bool,
}

impl Config {
    pub fn load(config_option: &mut ConfigOption) -> Result<Self, ConfigError> {
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
        let config_string = fs::read_to_string(config_file_path).map_err(ConfigError::Load)?;

        // Parse String to TOML String
        let config_file: ConfigOption =
            toml::from_str(&config_string).map_err(ConfigError::Parse)?;

        // Merge configs from CLI input
        let merged_config_option = config_file.merge(config_option);

        // Read signing key (TODO:)
        let signing_key_path = config_path.join(SIGNING_KEY_PATH);
        let signing_key = fs::read_to_string(signing_key_path).unwrap();

        Ok(Config {
            path: config_path,
            sequencer_rpc_url: merged_config_option.external_rpc_url.unwrap(),
            internal_rpc_url: merged_config_option.internal_rpc_url.unwrap(),
            cluster_rpc_url: merged_config_option.cluster_rpc_url.unwrap(),
            seeder_rpc_url: merged_config_option.seeder_rpc_url.unwrap(),
            distributed_key_generation_rpc_url: merged_config_option
                .distributed_key_generation_rpc_url
                .unwrap(),
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

    pub fn external_rpc_url(&self) -> &String {
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

    pub fn external_port(&self) -> Result<String, ConfigError> {
        let (_, port) = self
            .external_rpc_url()
            .split_once(':')
            .ok_or(ConfigError::InvalidExternalPort)?;

        Ok(port.to_owned())
    }

    pub fn cluster_port(&self) -> Result<String, ConfigError> {
        let (_, port) = self
            .cluster_rpc_url()
            .split_once(':')
            .ok_or(ConfigError::InvalidClusterPort)?;

        Ok(port.to_owned())
    }

    pub fn distributed_key_generation_rpc_url(&self) -> &String {
        &self.distributed_key_generation_rpc_url
    }

    pub fn signing_key(&self) -> &String {
        &self.signing_key
    }

    pub fn is_using_zkp(&self) -> bool {
        self.is_using_zkp
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Load(std::io::Error),
    Parse(toml::de::Error),
    RemoveConfigDirectory(std::io::Error),
    CreateConfigDirectory(std::io::Error),
    CreateConfigFile(std::io::Error),
    CreatePrivateKeyFile(std::io::Error),
    InvalidExternalPort,
    InvalidClusterPort,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ConfigError {}
