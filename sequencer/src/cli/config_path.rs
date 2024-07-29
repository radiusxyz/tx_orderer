use std::path::{Path, PathBuf};
use std::{env, fs};

use primitives::clap::{self, Parser};
use primitives::error::{Error, WrapError};
use primitives::serde::{Deserialize, Serialize};
use primitives::tracing;

use crate::cli::config_option::ConfigOption;
use crate::cli::error::ConfigError;
use crate::cli::SecureRpcEndpointConfigOption;

#[derive(Debug, Deserialize, Parser, Serialize)]
#[serde(crate = "primitives::serde")]
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
            fs::remove_dir_all(self)
                .map_err(|error| Error::new(ConfigError::RemoveConfigDirectory, error))
                .context(format_args!("{:?}", self))?;
        }

        // Create the directory.
        fs::create_dir_all(self)
            .map_err(|error| Error::new(ConfigError::CreateConfigDirectory, error))
            .context(format_args!("{:?}", self))?;

        // Create TOML config file.
        let toml_config_path = self.as_ref().join("config.toml");
        let toml_config_string = ConfigOption::default().get_toml_string();
        fs::write(toml_config_path, toml_config_string)
            .map_err(|error| Error::new(ConfigError::CreateConfigFile, error))
            .context(format_args!("{:?}", self))?;

        tracing::info!("Created a new config directory at {:?}", self.as_ref());
        Ok(())
    }

    pub fn init_secure_rpc_endpoint(&self) -> Result<(), Error> {
        // Remove the directory if it exists.
        if self.as_ref().exists() {
            fs::remove_dir_all(self)
                .map_err(|error| Error::new(ConfigError::RemoveConfigDirectory, error))
                .context(format_args!("{:?}", self))?;
        }

        // Create the directory.
        fs::create_dir_all(self)
            .map_err(|error| Error::new(ConfigError::CreateConfigDirectory, error))
            .context(format_args!("{:?}", self))?;

        // Create TOML config file.
        let toml_config_path = self.as_ref().join("config.toml");
        let toml_config_string = SecureRpcEndpointConfigOption::default().get_toml_string();
        fs::write(toml_config_path, toml_config_string)
            .map_err(|error| Error::new(ConfigError::CreateConfigFile, error))
            .context(format_args!("{:?}", self))?;

        tracing::info!("Created a new config directory at {:?}", self.as_ref());
        Ok(())
    }
}
