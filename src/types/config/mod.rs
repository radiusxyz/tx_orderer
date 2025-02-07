mod config_option;
mod config_path;

use std::{fs, path::PathBuf};

pub use config_option::*;
pub use config_path::*;
use serde::{Deserialize, Serialize};

pub const DEFAULT_DATA_PATH: &str = ".radius";
pub const DATABASE_DIR_NAME: &str = "database";
pub const LOG_DIR_NAME: &str = "logs";

pub const CONFIG_FILE_NAME: &str = "Config.toml";
pub const SIGNING_KEY_PATH: &str = "signing_key";
pub const DEFAULT_SIGNING_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub path: PathBuf,

    pub external_rpc_url: String,
    pub internal_rpc_url: String,
    pub cluster_rpc_url: String,

    pub seeder_rpc_url: String,

    pub distributed_key_generation_rpc_url: String,

    pub signing_key: String,

    pub is_using_zkp: bool,
}

/// Provides a default implementation for the `Config` struct.
///
/// This implementation is intended for testing and development purposes.
/// It initializes the configuration with preset values, including paths, RPC
/// URLs, and other parameters. These values are not suitable for production use
/// and should be explicitly overridden in a real-world deployment.
///
/// - `path`: Default directory for storing data.
/// - `external_rpc_url`: External RPC server address for external
///   communication.
/// - `internal_rpc_url`: Internal RPC server address for internal
///   communication.
/// - `cluster_rpc_url`: Address for cluster-related operations.
/// - `seeder_rpc_url`: Seeder service RPC address.
/// - `distributed_key_generation_rpc_url`: RPC address for distributed key
///   generation service.
/// - `signing_key`: A placeholder signing key for development.
/// - `is_using_zkp`: Boolean flag indicating whether Zero-Knowledge Proofs
///   (ZKP) are enabled.
///
/// Note: For production use, ensure these values are set explicitly in the
/// configuration file or environment variables to meet security and functional
/// requirements.
impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./data"),
            external_rpc_url: "http://127.0.0.1:3000".to_string(),
            internal_rpc_url: "http://127.0.0.1:4000".to_string(),
            cluster_rpc_url: "http://127.0.0.1:5000".to_string(),
            seeder_rpc_url: "http://127.0.0.1:6000".to_string(),
            distributed_key_generation_rpc_url: "http://127.0.0.1:7100".to_string(),
            signing_key: DEFAULT_SIGNING_KEY.to_string(),
            is_using_zkp: true,
        }
    }
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

        // Read signing key
        let signing_key_path = config_path.join(SIGNING_KEY_PATH);
        let signing_key = fs::read_to_string(signing_key_path).unwrap();

        Ok(Config {
            path: config_path,
            external_rpc_url: merged_config_option.external_rpc_url.unwrap(),
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

    pub fn database_path(&self) -> PathBuf {
        self.path.join(DATABASE_DIR_NAME)
    }

    pub fn log_path(&self) -> PathBuf {
        self.path.join(LOG_DIR_NAME)
    }

    pub fn external_port(&self) -> Result<String, ConfigError> {
        Ok(self
            .external_rpc_url
            .split(':')
            .last()
            .ok_or(ConfigError::InvalidExternalPort)?
            .to_string())
    }

    pub fn cluster_port(&self) -> Result<String, ConfigError> {
        Ok(self
            .cluster_rpc_url
            .split(':')
            .last()
            .ok_or(ConfigError::InvalidClusterPort)?
            .to_string())
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
