use std::{iter::zip, path::Path, str::FromStr};

use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, ProviderBuilder, RootProvider, WalletProvider,
    },
    signers::{k256::ecdsa::SigningKey, local::LocalSigner},
    transports::http::{Client, Http},
};
use chrono::Utc;
use DelegationManager::OperatorDetails;
use StakeRegistry::SignatureWithSaltAndExpiry;

use crate::avs::{seeder::SeederClient, types::*, Error, ErrorKind};

type EthereumProvider = FillProvider<
    JoinFill<
        JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

type SsalContract = Ssal::SsalInstance<
    Http<Client>,
    FillProvider<
        JoinFill<
            JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
            WalletFiller<EthereumWallet>,
        >,
        RootProvider<Http<Client>>,
        Http<Client>,
        Ethereum,
    >,
>;

type DelegationManagerContract = DelegationManager::DelegationManagerInstance<
    Http<Client>,
    FillProvider<
        JoinFill<
            JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
            WalletFiller<EthereumWallet>,
        >,
        RootProvider<Http<Client>>,
        Http<Client>,
        Ethereum,
    >,
>;

type StakeRegistryContract = StakeRegistry::StakeRegistryInstance<
    Http<Client>,
    FillProvider<
        JoinFill<
            JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
            WalletFiller<EthereumWallet>,
        >,
        RootProvider<Http<Client>>,
        Http<Client>,
        Ethereum,
    >,
>;

type AvsDirectoryContract = AvsDirectory::AvsDirectoryInstance<
    Http<Client>,
    FillProvider<
        JoinFill<
            JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
            WalletFiller<EthereumWallet>,
        >,
        RootProvider<Http<Client>>,
        Http<Client>,
        Ethereum,
    >,
>;

type AvsContract = Avs::AvsInstance<
    Http<Client>,
    FillProvider<
        JoinFill<
            JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
            WalletFiller<EthereumWallet>,
        >,
        RootProvider<Http<Client>>,
        Http<Client>,
        Ethereum,
    >,
>;

pub struct SsalClient {
    provider: EthereumProvider,
    signer: LocalSigner<SigningKey>,
    ssal_contract: SsalContract,
    delegation_manager_contract: DelegationManagerContract,
    stake_registry_contract: StakeRegistryContract,
    avs_directory_contract: AvsDirectoryContract,
    avs_contract: AvsContract,
    seeder_client: SeederClient,
}

unsafe impl Send for SsalClient {}

unsafe impl Sync for SsalClient {}

impl Clone for SsalClient {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            signer: self.signer.clone(),
            ssal_contract: self.ssal_contract.clone(),
            delegation_manager_contract: self.delegation_manager_contract.clone(),
            stake_registry_contract: self.stake_registry_contract.clone(),
            avs_directory_contract: self.avs_directory_contract.clone(),
            avs_contract: self.avs_contract.clone(),
            seeder_client: self.seeder_client.clone(),
        }
    }
}

impl SsalClient {
    pub fn new(
        ethereum_rpc_url: impl AsRef<str>,
        // keystore_path: impl AsRef<Path>,
        // keystore_password: impl AsRef<[u8]>,
        signing_key: impl AsRef<str>,
        ssal_contract_address: impl AsRef<str>,
        delegation_manager_contract_address: impl AsRef<str>,
        stake_registry_contract_address: impl AsRef<str>,
        avs_directory_contract_address: impl AsRef<str>,
        avs_contract_address: impl AsRef<str>,
        seeder_rpc_url: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let url = ethereum_rpc_url
            .as_ref()
            .parse()
            .map_err(|error| Error::boxed(ErrorKind::ParseRpcUrl, error))?;

        // let signer = LocalSigner::decrypt_keystore(keystore_path, keystore_password)
        //     .map_err(|error| (ErrorKind::Keystore, error))?;

        let signer = LocalSigner::from_str(signing_key.as_ref())
            .map_err(|error| (ErrorKind::ParseSigningKey, error))?;
        let wallet = EthereumWallet::new(signer.clone());

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);

        let ssal_contract_address = Address::from_str(ssal_contract_address.as_ref())
            .map_err(|error| (ErrorKind::ParseSsalContractAddress, error))?;
        let ssal_contract = Ssal::SsalInstance::new(ssal_contract_address, provider.clone());

        let delegation_manager_contract_address =
            Address::from_str(delegation_manager_contract_address.as_ref())
                .map_err(|error| (ErrorKind::ParseDelegationManagerContractAddress, error))?;
        let delegation_manager_contract =
            DelegationManager::new(delegation_manager_contract_address, provider.clone());

        let stake_registry_contract_address =
            Address::from_str(stake_registry_contract_address.as_ref())
                .map_err(|error| (ErrorKind::ParseStakeRegistryContractAddress, error))?;
        let stake_registry_contract =
            StakeRegistry::new(stake_registry_contract_address, provider.clone());

        let avs_directory_contract_address =
            Address::from_str(avs_directory_contract_address.as_ref())
                .map_err(|error| (ErrorKind::ParseAvsDirectoryContractAddress, error))?;
        let avs_directory_contract =
            AvsDirectory::new(avs_directory_contract_address, provider.clone());

        let avs_contract_address = Address::from_str(avs_contract_address.as_ref())
            .map_err(|error| (ErrorKind::ParseAvsContractAddress, error))?;
        let avs_contract = Avs::AvsInstance::new(avs_contract_address, provider.clone());

        let seeder_client = SeederClient::new(seeder_rpc_url)?;

        Ok(Self {
            provider,
            signer,
            ssal_contract,
            delegation_manager_contract,
            stake_registry_contract,
            avs_directory_contract,
            avs_contract,
            seeder_client,
        })
    }

    pub fn provider(&self) -> EthereumProvider {
        self.provider.clone()
    }

    pub fn address(&self) -> Address {
        self.provider.wallet().default_signer().address()
    }

    pub fn signer(&self) -> &LocalSigner<SigningKey> {
        &self.signer
    }

    pub async fn register_as_operator(&self) -> Result<(), Error> {
        let operator_details = OperatorDetails {
            earningsReceiver: self.address(),
            delegationApprover: Address::ZERO,
            stakerOptOutWindowBlocks: 0,
        };

        let _register_as_operator = self
            .delegation_manager_contract
            .registerAsOperator(operator_details, String::from(""))
            .send()
            .await
            .map_err(|error| (ErrorKind::RegisterAsOperator, error))?
            .get_receipt()
            .await
            .map_err(|error| (ErrorKind::RegisterAsOperator, error))?;

        let salt = FixedBytes::from_slice(&[0u8; 32]);
        let now = Utc::now().timestamp();
        let expiry: U256 = U256::from(now + 3600);
        let digest_hash = self
            .avs_directory_contract
            .calculateOperatorAVSRegistrationDigestHash(
                self.address(),
                *self.avs_contract.address(),
                salt,
                expiry,
            )
            .call()
            .await
            .map_err(|error| (ErrorKind::CalculateDigestHash, error))?
            ._0;

        let signature = self
            .signer()
            .sign_hash(&digest_hash)
            .await
            .map_err(|error| (ErrorKind::OperatorSignature, error))?;

        let operator_signature = SignatureWithSaltAndExpiry {
            signature: signature.as_bytes().into(),
            salt,
            expiry,
        };

        let register_operator_with_signature = self
            .stake_registry_contract
            .registerOperatorWithSignature(self.address(), operator_signature)
            .send()
            .await
            .map_err(|error| (ErrorKind::RegisterOperatorWithSignature, error))?
            .get_receipt()
            .await
            .map_err(|error| (ErrorKind::RegisterOperatorWithSignature, error))?;

        println!("{:?}", register_operator_with_signature);

        Ok(())
    }

    pub async fn initialize_cluster(
        &self,
        sequencer_rpc_url: impl AsRef<str>,
        sequencer_address: impl AsRef<str>,
        rollup_address: impl AsRef<str>,
    ) -> Result<(), Error> {
        let sequencer_rpc_url = sequencer_rpc_url.as_ref().to_owned();
        let sequencer_address = Address::from_str(sequencer_address.as_ref())
            .map_err(|error| (ErrorKind::ParseSequencerAddress, error))?;
        let rollup_address = Address::from_str(rollup_address.as_ref())
            .map_err(|error| (ErrorKind::ParseRollupAddress, error))?;

        self.seeder_client
            .register(sequencer_address, sequencer_rpc_url)
            .await?;

        let _transaction = self
            .ssal_contract
            .initializeCluster(sequencer_address, rollup_address)
            .send()
            .await
            .map_err(|error| (ErrorKind::InitializeCluster, error))?;

        Ok(())
    }

    pub async fn register_sequencer(
        &self,
        cluster_id: impl AsRef<str>,
        sequencer_address: impl AsRef<str>,
    ) -> Result<(), Error> {
        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterId, error))?;
        let sequencer_address = Address::from_str(sequencer_address.as_ref())
            .map_err(|error| (ErrorKind::ParseSequencerAddress, error))?;

        let _transaction = self
            .ssal_contract
            .registerSequencer(cluster_id, sequencer_address)
            .send()
            .await
            .map_err(|error| (ErrorKind::RegisterSequencer, error))?;

        Ok(())
    }

    pub async fn deregister_sequencer(
        &self,
        cluster_id: impl AsRef<str>,
        sequencer_address: impl AsRef<str>,
    ) -> Result<(), Error> {
        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterId, error))?;
        let sequencer_address = Address::from_str(sequencer_address.as_ref())
            .map_err(|error| (ErrorKind::ParseSequencerAddress, error))?;

        let _transaction = self
            .ssal_contract
            .deregisterSequencer(cluster_id, sequencer_address)
            .send()
            .await
            .map_err(|error| (ErrorKind::DeregisterSequencer, error))?;

        Ok(())
    }

    pub async fn register_block_commitment(
        &self,
        block_commitment: impl AsRef<str>,
        block_number: u64,
        rollup_id: u32,
        cluster_id: impl AsRef<str>,
    ) -> Result<(), Error> {
        let block_commitment = Bytes::from_str(block_commitment.as_ref())
            .map_err(|error| (ErrorKind::ParseBlockCommitment, error))?;

        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterId, error))?;

        let _transaction = self
            .avs_contract
            .createNewTask(block_commitment, block_number, rollup_id, cluster_id)
            .send()
            .await
            .map_err(|error| (ErrorKind::RegisterBlockCommitment, error))?;

        Ok(())
    }

    pub async fn respond_to_task(
        &self,
        task: Avs::Task,
        task_index: u32,
        block_commitment: impl AsRef<str>,
    ) -> Result<(), Error> {
        let block_commitment = Bytes::from_str(block_commitment.as_ref())
            .map_err(|error| (ErrorKind::ParseBlockCommitment, error))?;

        let message_hash = eip191_hash_message(block_commitment);

        // let signature = self
        //     .operator
        //     .sign_hash(&message_hash)
        //     .await
        //     .map_err(|error| (ErrorKind::SignTask, error))?;

        // let _transaction = self
        //     .avs_contract
        //     .respondToTask(task, task_index, signature.as_bytes().into())
        //     .send()
        //     .await
        //     .map_err(|error| (ErrorKind::RespondToTask, error))?;

        Ok(())
    }

    pub async fn get_sequencer_list(
        &self,
        cluster_id: impl AsRef<str>,
    ) -> Result<Vec<(Address, Option<String>)>, Error> {
        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterId, error))?;

        let sequencer_address_list: [Address; 30] = self
            .ssal_contract
            .getSequencers(cluster_id)
            .call()
            .await
            .map_err(|error| (ErrorKind::GetSequencerAddress, error))?
            ._0;

        // Filter sequencer address whose value is zero (== [0; 20])
        let sequencer_address_list: Vec<Address> = sequencer_address_list
            .into_iter()
            .filter(|sequencer_address| !sequencer_address.is_zero())
            .collect();

        let sequencer_rpc_url_list = self
            .seeder_client
            .get_sequencer_rpc_urls(sequencer_address_list.clone())
            .await?;

        let sequencer_list = zip(sequencer_address_list, sequencer_rpc_url_list).collect();

        Ok(sequencer_list)
    }
}
