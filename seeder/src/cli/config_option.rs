use std::{fs, path::PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};

use super::{ConfigPath, CONFIG_FILE_NAME};
use crate::error::Error;

const DEFAULT_SEEDER_RPC_URL: &str = "127.0.0.1:3000";
const DEFAULT_PROVIDER_WEBSOCKET_URL: &str = "ws://127.0.0.1:8545";
const DEFAULT_CONTRACT_ADDRESS: &str = "";

#[derive(Debug, Deserialize, Parser, Serialize)]
pub struct ConfigOption {
    #[doc = "Set the configuration file path to load from"]
    #[clap(long = "path")]
    pub path: Option<PathBuf>,

    #[doc = "Set the seeder rpc url"]
    #[clap(long = "seeder-rpc-url")]
    pub seeder_rpc_url: Option<String>,

    #[doc = "Set the provider websocket url"]
    #[clap(long = "provider-websocket-url")]
    pub provider_websocket_url: Option<String>,

    #[doc = "Set the contract address"]
    #[clap(long = "contract-address")]
    pub contract_address: Option<String>,
}

impl Default for ConfigOption {
    fn default() -> Self {
        Self {
            path: Some(ConfigPath::default().as_ref().into()),
            seeder_rpc_url: Some(DEFAULT_SEEDER_RPC_URL.into()),
            provider_websocket_url: Some(DEFAULT_PROVIDER_WEBSOCKET_URL.into()),
            contract_address: Some(DEFAULT_CONTRACT_ADDRESS.into()),
        }
    }
}

impl ConfigOption {
    pub fn load_config(config_option: &mut ConfigOption) -> Result<Self, Error> {
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
        let config_file: Self =
            toml::from_str(&config_string).map_err(|_| Error::ParseTomlString)?;

        // Merge configs from CLI input
        Ok(config_file.merge(config_option))
    }

    pub fn get_toml_string(&self) -> String {
        let mut toml_string = String::new();

        set_toml_comment(&mut toml_string, "Set seeder rpc url");
        set_toml_name_value(&mut toml_string, "seeder_rpc_url", &self.seeder_rpc_url);

        set_toml_comment(&mut toml_string, "Set provider websocket url");
        set_toml_name_value(
            &mut toml_string,
            "provider_websocket_url",
            &self.provider_websocket_url,
        );

        set_toml_comment(&mut toml_string, "Set contract address");
        set_toml_name_value(&mut toml_string, "contract_address", &self.contract_address);

        toml_string
    }

    pub fn merge(mut self, other: &ConfigOption) -> Self {
        if other.path.is_some() {
            self.path = other.path.clone();
        }

        if other.seeder_rpc_url.is_some() {
            self.seeder_rpc_url = other.seeder_rpc_url.clone();
        }

        if other.provider_websocket_url.is_some() {
            self.provider_websocket_url = other.provider_websocket_url.clone();
        }

        if other.contract_address.is_some() {
            self.contract_address = other.contract_address.clone();
        }

        self
    }
}

fn set_toml_comment(toml_string: &mut String, comment: &'static str) {
    let comment = format!("# {}\n", comment);

    toml_string.push_str(&comment);
}

fn set_toml_name_value<T>(toml_string: &mut String, name: &'static str, value: &Option<T>)
where
    T: std::fmt::Debug,
{
    let name_value = match value {
        Some(value) => format!("{} = {:?}\n\n", name, value),
        None => format!("# {} = {:?}\n\n", name, value),
    };

    toml_string.push_str(&name_value);
}
