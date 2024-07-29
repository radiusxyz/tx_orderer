use std::{
    env, fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{ConfigOption, CONFIG_FILE_NAME},
    error::Error,
};

#[derive(Debug, Deserialize, Parser, Serialize)]
pub struct ConfigPath {
    #[doc = "Set the sequencer configuration path"]
    #[clap(long = "path", default_value_t = Self::default().to_string())]
    path: String,
}

impl std::fmt::Display for ConfigPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl AsRef<Path> for ConfigPath {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl Default for ConfigPath {
    fn default() -> Self {
        let path = PathBuf::from(env::var("HOME").unwrap())
            .join(super::DEFAULT_HOME_PATH)
            .to_str()
            .unwrap()
            .to_string();

        Self { path }
    }
}

impl ConfigPath {
    pub fn init(&self) -> Result<(), Error> {
        // Remove the directory if it exists.
        if self.as_ref().exists() {
            fs::remove_dir_all(self).map_err(|_| Error::RemoveConfigDirectory)?;
        }

        // Create the directory
        fs::create_dir_all(self).map_err(|_| Error::CreateConfigDirectory)?;

        // Create config file
        let config_file_path = self.as_ref().join(CONFIG_FILE_NAME);
        let config_toml_string = ConfigOption::default().get_toml_string();
        fs::write(config_file_path, config_toml_string).map_err(|_| Error::CreateConfigFile)?;

        tracing::info!("Created a new config directory at {:?}", self.as_ref());
        Ok(())
    }
}
