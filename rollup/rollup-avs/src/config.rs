use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // Rollup
    pub database_path: PathBuf,
    pub block_creation_time: u64,

    // SSAL
    pub ssal_rpc_address: String,
    pub contract_address: String,
    pub cluster_id: [u8; 32],

    // Seeder
    pub seeder_rpc_address: String,
}

impl Config {
    pub fn load(config_path: impl AsRef<Path>) -> Result<Self, Error> {
        let config_string = fs::read_to_string(config_path)?;
        let config: Self = toml::from_str(&config_string)?;
        Ok(config)
    }
}
