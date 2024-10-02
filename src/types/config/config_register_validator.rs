use std::{
    env, fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    types::{ConfigOption, CONFIG_FILE_NAME, DEFAULT_SIGNING_KEY, SIGNING_KEY_PATH},
};

#[derive(Debug, Deserialize, Parser, Serialize)]
pub struct ConfigRegisterValidator {
    // #[clap(long = "ethereum_rpc_url", default_value_t = String::from("https://ethereum-holesky-rpc.publicnode.com"))]
    #[clap(long = "ethereum_rpc_url", default_value_t = String::from("http://localhost:8545"))]
    ethereum_rpc_url: String,

    // #[clap(long = "signing_key", default_value_t =
    // String::from("d381db1707bdc54dc91842b189834719ae6ec92534aaf9e607ebb2bcf3583157"))]
    #[clap(long = "signing_key", default_value_t = String::from("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"))]
    signing_key: String,

    // #[clap(long = "delegation_manager_contract_address", default_value_t =
    // String::from("0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e"))]
    // #[clap(long = "delegation_manager_contract_address", default_value_t =
    // String::from("0xA44151489861Fe9e3055d95adC98FbD462B948e7"))]

    // "delegation": "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9",
    // "delegationImplementation": "0xB7f8BC63BbcaD18155201308C8f3540b07f84F5e",
    #[clap(long = "delegation_manager_contract_address", default_value_t = String::from("0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9"))]
    delegation_manager_contract_address: String,

    // #[clap(long = "avs_directory_contract_address", default_value_t =
    // String::from("0x055733000064333CaDDbC92763c58BF0192fFeBf"))]

    // "avsDirectory": "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707",
    // "avsDirectoryImplementation": "0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82",
    #[clap(long = "avs_directory_contract_address", default_value_t = String::from("0x5FC8d32690cc91D4c39d9d3abcBD16989F875707"))]
    avs_directory_contract_address: String,

    // #[clap(long = "stake_registry_contract_address", default_value_t =
    // String::from("0x12B6bf07dFA1a62a1521f59bdC65019234884315"))]
    // #[clap(long = "stake_registry_contract_address", default_value_t =
    // String::from("0xf4d28815707ACBDb020F3a61c77F6B64DB0B2936"))]

    // "ECDSAStakeRegistry": "0x9E545E3C0baAB3E08CdfD552C960A1050f373042",
    // "ECDSAStakeRegistryImplementation": "0xa82fF9aFd8f496c3d6ac40E2a0F282E47488CFc9",
    #[clap(long = "stake_registry_contract_address", default_value_t = String::from("0x9E545E3C0baAB3E08CdfD552C960A1050f373042"))]
    stake_registry_contract_address: String,

    // #[clap(long = "avs_contract_address", default_value_t =
    // String::from("0xf5059a5d33d5853360d16c683c16e67980206f36"))]
    // #[clap(long = "avs_contract_address", default_value_t =
    // String::from("0x3334233bb2B353787f7a19b351C006e776F4564A"))]

    // "SequencingServiceManagerImplementation": "0xf5059a5D33d5853360D16C683c16e67980206f36",
    // "SequencingServiceManagerProxy": "0x84eA74d481Ee0A5332c457a4d796187F6Ba67fEB",
    #[clap(long = "avs_contract_address", default_value_t = String::from("0x84eA74d481Ee0A5332c457a4d796187F6Ba67fEB"))]
    avs_contract_address: String,
}

impl ConfigRegisterValidator {
    pub async fn init(&self) {
        use radius_sequencer_sdk::validation_eigenlayer::publisher::Publisher;

        let client = Publisher::new(
            &self.ethereum_rpc_url,
            &self.signing_key,
            &self.delegation_manager_contract_address,
            &self.avs_directory_contract_address,
            &self.stake_registry_contract_address,
            &self.avs_contract_address,
        )
        .unwrap();

        println!("Address: {:?}", client.address());

        if !client.is_operator().await.unwrap() {
            let transaction_hash = client.register_as_operator().await.unwrap();

            println!(
                "Operator registered - transaction_hash: {:?}",
                transaction_hash
            );
        }

        println!("Address: {:?}", client.address());

        if !client.is_operator_registered_on_avs().await.unwrap() {
            let transaction_hash = client.register_operator_on_avs().await.unwrap();

            println!(
                "Operator registered on AVS - transaction_hash: {:?}",
                transaction_hash
            );
        }
    }
}
