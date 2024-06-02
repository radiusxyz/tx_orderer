use std::{
    fs,
    path::{Path, PathBuf},
};

use sequencer_framework::serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Deserialize, Serialize)]
#[serde(crate = "sequencer_framework::serde")]
pub struct Config {
    database_path: PathBuf,
    seeder_address: String,
}

impl Config {
    pub fn load(config_path: impl AsRef<Path>) -> Result<Self, Error> {
        let config_string = fs::read_to_string(config_path)?;
        let config: Self = toml::from_str(&config_string)?;
        Ok(config)
    }

    pub fn database_path(&self) -> &PathBuf {
        &self.database_path
    }

    pub fn seeder_address(&self) -> &String {
        &self.seeder_address
    }
}
