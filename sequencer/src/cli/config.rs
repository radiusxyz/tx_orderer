use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{ConfigOption, ConfigPath, CONFIG_FILE_NAME};
use crate::{error::Error, types::ClusterType};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    path: PathBuf,
    sequencer_rpc_url: String,
    internal_rpc_url: String,
    cluster_rpc_url: String,
    provider_rpc_url: String,
    provider_websocket_url: String,
    cluster_type: ClusterType,
    liveness_contract_address: Option<String>,
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

        match merged_config_option.cluster_type.unwrap().as_str() {
            "local" => Ok(Config {
                path: config_path,
                sequencer_rpc_url: merged_config_option.sequencer_rpc_url.unwrap(),
                internal_rpc_url: merged_config_option.internal_rpc_url.unwrap(),
                cluster_rpc_url: merged_config_option.cluster_rpc_url.unwrap(),
                provider_rpc_url: merged_config_option.provider_rpc_url.unwrap(),
                provider_websocket_url: merged_config_option.provider_websocket_url.unwrap(),
                liveness_contract_address: None,
                cluster_type: ClusterType::Local,
            }),
            "eigen_layer" => Ok(Config {
                path: config_path,
                sequencer_rpc_url: merged_config_option.sequencer_rpc_url.unwrap(),
                internal_rpc_url: merged_config_option.internal_rpc_url.unwrap(),
                cluster_rpc_url: merged_config_option.cluster_rpc_url.unwrap(),
                provider_rpc_url: merged_config_option.provider_rpc_url.unwrap(),
                provider_websocket_url: merged_config_option.provider_websocket_url.unwrap(),
                liveness_contract_address: Some(
                    merged_config_option.liveness_contract_address.unwrap(),
                ),
                cluster_type: ClusterType::EigenLayer,
            }),
            _ => Err(Error::InvalidClusterType),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn database_path(&self) -> PathBuf {
        self.path.join("database")
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

    pub fn provider_rpc_url(&self) -> &String {
        &self.provider_rpc_url
    }

    pub fn provider_websocket_url(&self) -> &String {
        &self.provider_websocket_url
    }

    pub fn liveness_contract_address(&self) -> &Option<String> {
        &self.liveness_contract_address
    }

    pub fn cluster_type(&self) -> &ClusterType {
        &self.cluster_type
    }
}
