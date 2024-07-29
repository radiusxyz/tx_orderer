use std::fs;
use std::path::PathBuf;

use primitives::clap::{self, Parser};
use primitives::error::{Error, WrapError};
use primitives::serde::{Deserialize, Serialize};
use primitives::toml;
use signer::generate_private_key;

use crate::cli::config_path::ConfigPath;
use crate::cli::error::ConfigError;

#[derive(Debug, Deserialize, Parser, Serialize)]
#[serde(crate = "primitives::serde")]
pub struct ConfigOption {
    #[doc = "Set the configuration file path to load from"]
    #[clap(long = "path")]
    pub path: Option<PathBuf>,

    #[doc = "Set JSON-RPC address"]
    #[clap(long = "rpc-address")]
    pub rpc_endpoint: Option<String>,

    #[doc = "Set Raft address"]
    #[clap(long = "raft-address")]
    pub raft_endpoint: Option<String>,

    #[doc = "(Optional) Set the peer address"]
    #[clap(long = "raft-peer-address")]
    pub raft_peer_endpoint: Option<String>,

    #[doc = "Set decryptors"]
    #[clap(long = "decryptor", num_args = 1.., value_delimiter = ' ')]
    pub decryptor: Option<Vec<String>>,

    #[doc = "Set white-list"]
    #[clap(long = "white-list", num_args = 1.., value_delimiter = ' ')]
    pub white_list: Option<Vec<String>>,

    #[doc = "Set the number of async threads for IO-bound tasks"]
    #[clap(long = "async-thread-count")]
    pub async_thread_count: Option<usize>,

    #[doc = "Set the maximum number of subprocess a sequencer can spawn"]
    #[clap(long = "process-count")]
    pub process_count: Option<usize>,

    #[doc = "Set the number of worker threads for CPU-bound tasks"]
    #[clap(long = "worker-thread-count")]
    pub worker_thread_count: Option<usize>,

    #[doc = "Set the size of the worker queue to which CPU-bound tasks are passed"]
    #[clap(long = "work-queue-capacity")]
    pub work_queue_capacity: Option<usize>,

    #[doc = "Set using zkp"]
    #[clap(long = "using-zkp")]
    pub do_verify_tx_with_zkp: Option<bool>,

    #[doc = "Set block generation time"]
    #[clap(long = "block-generation-time")]
    pub block_generation_time: Option<u64>,
}

impl Default for ConfigOption {
    fn default() -> Self {
        Self {
            path: Some(ConfigPath::default().as_ref().into()),
            rpc_endpoint: Some("0.0.0.0:8000".into()),
            raft_endpoint: Some("127.0.0.1:9000".into()),
            raft_peer_endpoint: None,
            decryptor: Some(vec!["127.0.0.1:8000".into()]),
            white_list: Some(vec!["127.0.0.1:9000".into()]),
            async_thread_count: Some(4),
            process_count: Some(8),
            worker_thread_count: Some(4),
            work_queue_capacity: Some(16),
            do_verify_tx_with_zkp: Some(false),
            block_generation_time: Some(5),
        }
    }
}

impl ConfigOption {
    pub fn load_config(config_option: &mut Box<Self>) -> Result<Self, Error> {
        let config_path = match config_option.path.as_mut() {
            Some(config_path) => config_path.clone(),
            None => {
                let config_path: PathBuf = ConfigPath::default().as_ref().into();
                config_option.path.replace(config_path.clone());
                config_path
            }
        };

        // Read file from to String.
        let config_file_path = config_path.join("config.toml");
        let config_string = fs::read_to_string(&config_file_path)
            .map_err(|error| Error::new(ConfigError::LoadConfigOption, error))
            .context(format_args!("{:?}", config_file_path))?;

        // Parse String to TOML String.
        let config_file: Self = toml::from_str(&config_string)
            .map_err(|error| Error::new(ConfigError::ParseTomlString, error))?;

        // Generate a private key.
        let private_key_path = config_path.join("private_key");
        let private_key = generate_private_key()?.into_inner();
        fs::write(&private_key_path, private_key)
            .map_err(|error| Error::new(ConfigError::CreatePrivateKeyFile, error))
            .context(format_args!("{:?}", private_key_path))?;

        // Merge configs from CLI input.
        Ok(config_file.merge(config_option))
    }

    fn toml_comment(toml_string: &mut String, comment: &'static str) {
        let comment = format!("# {}\n", comment);

        toml_string.push_str(&comment);
    }

    fn toml_name_value<T>(toml_string: &mut String, name: &'static str, value: &Option<T>)
    where
        T: std::fmt::Debug,
    {
        let name_value = match value {
            Some(value) => format!("{} = {:?}\n\n", name, value),
            None => format!("# {} = {:?}\n\n", name, value),
        };

        toml_string.push_str(&name_value);
    }

    pub fn get_toml_string(&self) -> String {
        let mut toml_string = String::new();

        Self::toml_comment(&mut toml_string, "Set JSON-RPC address");
        Self::toml_name_value(&mut toml_string, "rpc_endpoint", &self.rpc_endpoint);

        Self::toml_comment(&mut toml_string, "Set Raft address");
        Self::toml_name_value(&mut toml_string, "raft_endpoint", &self.raft_endpoint);

        Self::toml_comment(&mut toml_string, "(Optional) Set the peer address");
        Self::toml_name_value(
            &mut toml_string,
            "raft_peer_endpoint",
            &self.raft_peer_endpoint,
        );

        Self::toml_comment(&mut toml_string, "Set decryptors");
        Self::toml_name_value(&mut toml_string, "decryptor", &self.decryptor);

        Self::toml_comment(&mut toml_string, "Set white-list");
        Self::toml_name_value(&mut toml_string, "white_list", &self.white_list);

        Self::toml_comment(
            &mut toml_string,
            "Set the number of async threads for IO-bound tasks",
        );
        Self::toml_name_value(
            &mut toml_string,
            "async_thread_count",
            &self.async_thread_count,
        );

        Self::toml_comment(
            &mut toml_string,
            "Set the maximum number of subprocess a sequencer can spawn",
        );
        Self::toml_name_value(&mut toml_string, "process_count", &self.process_count);

        Self::toml_comment(
            &mut toml_string,
            "Set the number of worker threads for CPU-bound tasks",
        );
        Self::toml_name_value(
            &mut toml_string,
            "worker_thread_count",
            &self.worker_thread_count,
        );

        Self::toml_comment(
            &mut toml_string,
            "Set the size of the worker queue to which CPU-bound tasks are passed",
        );
        Self::toml_name_value(
            &mut toml_string,
            "work_queue_capacity",
            &self.work_queue_capacity,
        );

        Self::toml_comment(&mut toml_string, "(Optional) Set using zkp");
        Self::toml_name_value(
            &mut toml_string,
            "do_verify_tx_with_zkp",
            &self.do_verify_tx_with_zkp,
        );

        Self::toml_comment(&mut toml_string, "Set block generation time");
        Self::toml_name_value(
            &mut toml_string,
            "block_generation_time",
            &self.block_generation_time,
        );

        toml_string
    }

    fn merge(mut self, other: &mut Box<Self>) -> Self {
        if other.path.is_some() {
            self.path = other.path.take();
        }

        if other.rpc_endpoint.is_some() {
            self.rpc_endpoint = other.rpc_endpoint.take();
        }

        if other.raft_endpoint.is_some() {
            self.raft_endpoint = other.raft_endpoint.take();
        }

        if other.raft_peer_endpoint.is_some() {
            self.raft_peer_endpoint = other.raft_peer_endpoint.take();
        }

        if other.decryptor.is_some() {
            self.decryptor = other.decryptor.take();
        }

        if other.white_list.is_some() {
            self.white_list = other.white_list.take();
        }

        if other.async_thread_count.is_some() {
            self.async_thread_count = other.async_thread_count.take();
        }

        if other.process_count.is_some() {
            self.process_count = other.process_count.take();
        }

        if other.worker_thread_count.is_some() {
            self.worker_thread_count = other.worker_thread_count.take();
        }

        if other.work_queue_capacity.is_some() {
            self.work_queue_capacity = other.work_queue_capacity.take();
        }

        if other.do_verify_tx_with_zkp.is_some() {
            self.do_verify_tx_with_zkp = other.do_verify_tx_with_zkp.take();
        }

        if other.block_generation_time.is_some() {
            self.block_generation_time = other.block_generation_time.take();
        }

        self
    }
}
