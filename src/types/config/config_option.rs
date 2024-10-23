use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

use super::ConfigPath;

const DEFAULT_EXTERNAL_RPC_URL: &str = "http://127.0.0.1:3000"; // external rpc url
const DEFAULT_INTERNAL_RPC_URL: &str = "http://127.0.0.1:4000";
// TODO: temporary use external rpc url
const DEFAULT_CLUSTER_RPC_URL: &str = "http://127.0.0.1:3000";

const DEFAULT_SEEDER_RPC_URL: &str = "http://127.0.0.1:6000";

const DEFAULT_KEY_MANAGEMENT_SYSTEM_RPC_URL: &str = "http://127.0.0.1:7100";

// const DEFAULT_CLUSTER_TYPE: &str = "local";

// const DEFAULT_LIVENESS_PROVIDER_RPC_URL: &str = "http://127.0.0.1:8545";
// const DEFAULT_LIVENESS_PROVIDER_WEBSOCKET_URL: &str = "ws://127.0.0.1:8545";
// const DEFAULT_LIVENESS_CONTRACT_ADDRESS: &str = "";

#[derive(Debug, Deserialize, Parser, Serialize)]
pub struct ConfigOption {
    #[doc = "Set the configuration file path to load from"]
    #[clap(long = "path")]
    pub path: Option<PathBuf>,

    #[doc = "Set the external rpc url"]
    #[clap(long = "external-rpc-url")]
    pub external_rpc_url: Option<String>,

    #[doc = "Set the internal rpc url"]
    #[clap(long = "internal-rpc-url")]
    pub internal_rpc_url: Option<String>,

    #[doc = "Set the cluster rpc url"]
    #[clap(long = "cluster-rpc-url")]
    pub cluster_rpc_url: Option<String>,

    #[doc = "Set the seeder rpc url"]
    #[clap(long = "seeder-rpc-url")]
    pub seeder_rpc_url: Option<String>,

    #[doc = "Set the key management system rpc url"]
    #[clap(long = "key-management-system-rpc-url")]
    pub key_management_system_rpc_url: Option<String>,

    #[doc = "Set using zkp"]
    #[clap(long = "is-using-zkp")]
    pub is_using_zkp: Option<bool>,
}

impl Default for ConfigOption {
    fn default() -> Self {
        Self {
            path: Some(ConfigPath::default().as_ref().into()),

            external_rpc_url: Some(DEFAULT_EXTERNAL_RPC_URL.into()),
            internal_rpc_url: Some(DEFAULT_INTERNAL_RPC_URL.into()),
            cluster_rpc_url: Some(DEFAULT_CLUSTER_RPC_URL.into()),

            seeder_rpc_url: Some(DEFAULT_SEEDER_RPC_URL.into()),
            key_management_system_rpc_url: Some(DEFAULT_KEY_MANAGEMENT_SYSTEM_RPC_URL.into()),

            is_using_zkp: Some(false),
        }
    }
}

impl ConfigOption {
    pub fn get_toml_string(&self) -> String {
        let mut toml_string = String::new();

        set_toml_comment(&mut toml_string, "Set sequencer rpc url");
        set_toml_name_value(&mut toml_string, "external_rpc_url", &self.external_rpc_url);

        set_toml_comment(&mut toml_string, "Set internal rpc url");
        set_toml_name_value(&mut toml_string, "internal_rpc_url", &self.internal_rpc_url);

        set_toml_comment(&mut toml_string, "Set cluster rpc url");
        set_toml_name_value(&mut toml_string, "cluster_rpc_url", &self.cluster_rpc_url);

        set_toml_comment(&mut toml_string, "Set seeder rpc url");
        set_toml_name_value(&mut toml_string, "seeder_rpc_url", &self.seeder_rpc_url);

        set_toml_comment(&mut toml_string, "Set key management system rpc url");
        set_toml_name_value(
            &mut toml_string,
            "key_management_system_rpc_url",
            &self.key_management_system_rpc_url,
        );

        set_toml_comment(&mut toml_string, "Set using zkp");
        set_toml_name_value(&mut toml_string, "is_using_zkp", &self.is_using_zkp);

        toml_string
    }

    pub fn merge(mut self, other: &ConfigOption) -> Self {
        if other.path.is_some() {
            self.path.clone_from(&other.path);
        }

        if other.external_rpc_url.is_some() {
            self.external_rpc_url.clone_from(&other.external_rpc_url);
        }

        if other.internal_rpc_url.is_some() {
            self.internal_rpc_url.clone_from(&other.internal_rpc_url);
        }

        if other.cluster_rpc_url.is_some() {
            self.cluster_rpc_url.clone_from(&other.cluster_rpc_url);
        }

        if other.seeder_rpc_url.is_some() {
            self.seeder_rpc_url.clone_from(&other.seeder_rpc_url)
        }

        if other.key_management_system_rpc_url.is_some() {
            self.key_management_system_rpc_url
                .clone_from(&other.key_management_system_rpc_url);
        }

        if other.is_using_zkp.is_some() {
            self.is_using_zkp.clone_from(&other.is_using_zkp);
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
