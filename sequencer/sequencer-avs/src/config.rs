use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // Database
    pub database_path: PathBuf,

    // IPs
    pub sequencer_rpc_address: String,
    pub ssal_address: String,
    pub seeder_address: String,

    // SSAL-related
    pub sequencer_public_key: String,
    pub contract_address: String,
    pub cluster_id: [u8; 32],
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let config_string = fs::read_to_string(path).map_err(Error::OpenConfig)?;
        let config: Self = toml::from_str(&config_string).map_err(Error::ParseConfig)?;
        Ok(config)
    }
}
