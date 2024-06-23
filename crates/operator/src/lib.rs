use std::str::FromStr;

use alloy_network::EthereumSigner;
use alloy_primitives::{Address, FixedBytes, Signature, U256};
use alloy_provider::ProviderBuilder;
use alloy_signer::{k256::ecdsa::SigningKey, Signer};
use alloy_signer_wallet::{LocalWallet, Wallet};
use chrono::Utc;
use eigen_client_elcontracts::{
    reader::ELChainReader,
    writer::{ELChainWriter, Operator},
};
use eigen_utils::binding::ECDSAStakeRegistry::{self, SignatureWithSaltAndExpiry};
use rand::RngCore;

pub struct EigenLayerOperator {
    wallet: Wallet<SigningKey>,
}

impl Clone for EigenLayerOperator {
    fn clone(&self) -> Self {
        Self {
            wallet: self.wallet.clone(),
        }
    }
}

impl EigenLayerOperator {
    pub async fn register(
        ethereum_rpc_url: impl AsRef<str>,
        signing_key: impl AsRef<str>,
        avs_contract_address: impl AsRef<str>,
        avs_directory_contract_address: impl AsRef<str>,
        delegation_manager_contract_address: impl AsRef<str>,
        stake_registry_contract_address: impl AsRef<str>,
        is_registered: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let wallet = LocalWallet::from_str(signing_key.as_ref())?;

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .signer(EthereumSigner::from(wallet.clone()))
            .on_http(ethereum_rpc_url.as_ref().parse()?);

        let avs_contract_address = Address::from_str(avs_contract_address.as_ref())?;
        let avs_directory_contract_address =
            Address::from_str(avs_directory_contract_address.as_ref())?;
        let delegation_manager_contract_address =
            Address::from_str(delegation_manager_contract_address.as_ref())?;
        let stake_registry_contract_address =
            Address::from_str(stake_registry_contract_address.as_ref())?;

        let default_slasher = Address::ZERO; // We don't need slasher for our example.
        let default_strategy = Address::ZERO; // We don't need strategy for our example.
        let elcontracts_reader_instance = ELChainReader::new(
            default_slasher,
            delegation_manager_contract_address,
            avs_directory_contract_address,
            ethereum_rpc_url.as_ref().to_owned(),
        );
        let elcontracts_writer_instance = ELChainWriter::new(
            delegation_manager_contract_address,
            default_strategy,
            elcontracts_reader_instance.clone(),
            ethereum_rpc_url.as_ref().to_owned(),
            signing_key.as_ref().to_owned(),
        );

        let operator = Operator::new(
            wallet.address(),
            wallet.address(),
            Address::ZERO,
            0u32,
            None,
        );

        // In case you are running holesky. Comment the below register_as_operator call after the first
        // call . Since we can register only once per operator.
        if !is_registered {
            let _tx_hash = elcontracts_writer_instance
                .register_as_operator(operator)
                .await?;
        }

        println!("Registered successfully..");

        let mut salt = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut salt);
        let salt = FixedBytes::from_slice(&salt);
        let now = Utc::now().timestamp();
        let expiry: U256 = U256::from(now + 3600);
        let digest_hash = elcontracts_reader_instance
            .calculate_operator_avs_registration_digest_hash(
                wallet.address(),
                avs_contract_address,
                salt,
                expiry,
            )
            .await?;

        let signature = wallet.sign_hash(&digest_hash).await?;

        let operator_signature = SignatureWithSaltAndExpiry {
            signature: signature.as_bytes().into(),
            salt,
            expiry: expiry,
        };

        let contract_ecdsa_stake_registry =
            ECDSAStakeRegistry::new(stake_registry_contract_address, provider);

        // If you wish to run on holesky, please deploy the stake registry contract(it's not deployed right now)
        // and uncomment the gas and gas_price
        let registeroperator_details = contract_ecdsa_stake_registry
            .registerOperatorWithSignature(wallet.clone().address(), operator_signature);
        let _tx = registeroperator_details
            // .gas(300000)
            // .gas_price(20000000000)
            .send()
            .await?
            .get_receipt()
            .await?;

        Ok(Self { wallet })
    }

    pub async fn sign_hash(
        &self,
        hash: &FixedBytes<32>,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let signature = self.wallet.sign_hash(hash).await?;
        Ok(signature)
    }
}
