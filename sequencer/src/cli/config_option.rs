use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

use super::ConfigPath;

const DEFAULT_SEQUENCER_RPC_URL: &str = "127.0.0.1:3000";
const DEFAULT_INTERNAL_RPC_URL: &str = "127.0.0.1:3001";
const DEFAULT_CLUSTER_RPC_URL: &str = "127.0.0.1:3002";
const DEFAULT_PROVIDER_RPC_URL: &str = "http://127.0.0.1:8545";
const DEFAULT_PROVIDER_WEBSOCKET_URL: &str = "ws://127.0.0.1:8545";
const DEFAULT_LIVENESS_CONTRACT_ADDRESS: &str = "";
const DEFAULT_CLUSTER_TYPE: &str = "local";

#[derive(Debug, Deserialize, Parser, Serialize)]
pub struct ConfigOption {
    #[doc = "Set the configuration file path to load from"]
    #[clap(long = "path")]
    pub path: Option<PathBuf>,

    #[doc = "Set the sequencer rpc url"]
    #[clap(long = "sequencer-rpc-url")]
    pub sequencer_rpc_url: Option<String>,

    #[doc = "Set the internal rpc url"]
    #[clap(long = "internal-rpc-url")]
    pub internal_rpc_url: Option<String>,

    #[doc = "Set the cluster rpc url"]
    #[clap(long = "cluster-rpc-url")]
    pub cluster_rpc_url: Option<String>,

    #[doc = "Set the liveness provider rpc url"]
    #[clap(long = "liveness-provider-rpc-url")]
    pub liveness_provider_rpc_url: Option<String>,

    #[doc = "Set the liveness provider websocket url"]
    #[clap(long = "liveness-provider-websocket-url")]
    pub liveness_provider_websocket_url: Option<String>,

    #[doc = "Set the liveness contract address"]
    #[clap(long = "liveness-contract-address")]
    pub liveness_contract_address: Option<String>,

    #[doc = "Set the cluster types"]
    #[clap(long = "cluster-types")]
    pub cluster_type: Option<String>,
}

impl Default for ConfigOption {
    fn default() -> Self {
        Self {
            path: Some(ConfigPath::default().as_ref().into()),
            sequencer_rpc_url: Some(DEFAULT_SEQUENCER_RPC_URL.into()),
            internal_rpc_url: Some(DEFAULT_INTERNAL_RPC_URL.into()),
            cluster_rpc_url: Some(DEFAULT_CLUSTER_RPC_URL.into()),
            liveness_provider_rpc_url: Some(DEFAULT_PROVIDER_RPC_URL.into()),
            liveness_provider_websocket_url: Some(DEFAULT_PROVIDER_WEBSOCKET_URL.into()),
            liveness_contract_address: Some(DEFAULT_LIVENESS_CONTRACT_ADDRESS.into()),
            cluster_type: Some(DEFAULT_CLUSTER_TYPE.into()),
        }
    }
}

impl ConfigOption {
    pub fn get_toml_string(&self) -> String {
        let mut toml_string = String::new();

        set_toml_comment(&mut toml_string, "Set sequencer rpc url");
        set_toml_name_value(
            &mut toml_string,
            "sequencer_rpc_url",
            &self.sequencer_rpc_url,
        );

        set_toml_comment(&mut toml_string, "Set internal rpc url");
        set_toml_name_value(&mut toml_string, "internal_rpc_url", &self.internal_rpc_url);

        set_toml_comment(&mut toml_string, "Set cluster rpc url");
        set_toml_name_value(&mut toml_string, "cluster_rpc_url", &self.cluster_rpc_url);

        set_toml_comment(&mut toml_string, "Set liveness provider rpc url");
        set_toml_name_value(
            &mut toml_string,
            "liveness_provider_rpc_url",
            &self.liveness_provider_rpc_url,
        );

        set_toml_comment(&mut toml_string, "Set liveness provider websocket url");
        set_toml_name_value(
            &mut toml_string,
            "liveness_provider_websocket_url",
            &self.liveness_provider_websocket_url,
        );

        set_toml_comment(&mut toml_string, "Set liveness contract address");
        set_toml_name_value(
            &mut toml_string,
            "liveness_contract_address",
            &self.liveness_contract_address,
        );

        set_toml_comment(&mut toml_string, "Set cluster type");
        set_toml_name_value(&mut toml_string, "cluster_type", &self.cluster_type);

        toml_string
    }

    pub fn merge(mut self, other: &ConfigOption) -> Self {
        if other.path.is_some() {
            self.path = other.path.clone();
        }

        if other.sequencer_rpc_url.is_some() {
            self.sequencer_rpc_url = other.sequencer_rpc_url.clone();
        }

        if other.internal_rpc_url.is_some() {
            self.internal_rpc_url = other.internal_rpc_url.clone();
        }

        if other.cluster_rpc_url.is_some() {
            self.cluster_rpc_url = other.cluster_rpc_url.clone();
        }

        if other.liveness_provider_rpc_url.is_some() {
            self.liveness_provider_rpc_url = other.liveness_provider_rpc_url.clone();
        }

        if other.liveness_provider_websocket_url.is_some() {
            self.liveness_provider_websocket_url = other.liveness_provider_websocket_url.clone();
        }

        if other.liveness_contract_address.is_some() {
            self.liveness_contract_address = other.liveness_contract_address.clone();
        }

        if other.cluster_type.is_some() {
            self.cluster_type = other.cluster_type.clone();
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
