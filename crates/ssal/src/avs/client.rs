use std::{iter::zip, str::FromStr, sync::Arc};

use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, Provider, ProviderBuilder, RootProvider, WalletProvider,
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
    inner: Arc<SsalClientInner>,
}

struct SsalClientInner {
    provider: EthereumProvider,
    signer: LocalSigner<SigningKey>,
    ssal_contract: SsalContract,
    seeder_client: SeederClient,
    delegation_manager_contract: DelegationManagerContract,
    stake_registry_contract: StakeRegistryContract,
    avs_directory_contract: AvsDirectoryContract,
    avs_contract: AvsContract,
}

unsafe impl Send for SsalClient {}

unsafe impl Sync for SsalClient {}

impl Clone for SsalClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl SsalClient {
    pub fn new(
        ethereum_rpc_url: impl AsRef<str>,
        signing_key: impl AsRef<str>,
        seeder_rpc_url: impl AsRef<str>,
        ssal_contract_address: impl AsRef<str>,
        delegation_manager_contract_address: impl AsRef<str>,
        stake_registry_contract_address: impl AsRef<str>,
        avs_directory_contract_address: impl AsRef<str>,
        avs_contract_address: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let url = ethereum_rpc_url
            .as_ref()
            .parse()
            .map_err(|error| Error::boxed(ErrorKind::ParseRpcUrl, error))?;

        let signer = LocalSigner::from_str(signing_key.as_ref())
            .map_err(|error| (ErrorKind::ParseSigningKey, error))?;

        let wallet = EthereumWallet::new(signer.clone());

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);

        let seeder_client = SeederClient::new(seeder_rpc_url)?;

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

        let inner = SsalClientInner {
            provider,
            signer,
            ssal_contract,
            seeder_client,
            delegation_manager_contract,
            stake_registry_contract,
            avs_directory_contract,
            avs_contract,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn provider(&self) -> EthereumProvider {
        self.inner.provider.clone()
    }

    pub fn address(&self) -> Address {
        self.inner.provider.wallet().default_signer().address()
    }

    pub fn signer(&self) -> &LocalSigner<SigningKey> {
        &self.inner.signer
    }

    pub async fn get_block_number(&self) -> Result<u64, Error> {
        self.inner
            .provider
            .get_block_number()
            .await
            .map_err(|error| (ErrorKind::GetBlockNumber, error).into())
    }

    pub async fn is_operator(&self) -> Result<bool, Error> {
        let is_operator = self
            .inner
            .delegation_manager_contract
            .isOperator(self.address())
            .call()
            .await
            .map_err(|error| (ErrorKind::IsOperator, error))?
            ._0;

        Ok(is_operator)
    }

    pub async fn register_as_operator(&self) -> Result<(), Error> {
        let operator_details = OperatorDetails {
            earningsReceiver: self.address(),
            delegationApprover: Address::ZERO,
            stakerOptOutWindowBlocks: 0,
        };

        let register_as_operator = self
            .inner
            .delegation_manager_contract
            .registerAsOperator(operator_details, String::from(""))
            .send()
            .await
            .map_err(|error| (ErrorKind::RegisterAsOperator, error))?
            .get_receipt()
            .await
            .map_err(|error| (ErrorKind::RegisterAsOperator, error))?;

        println!("{:?}", register_as_operator.block_number);
        println!("{:?}", register_as_operator.transaction_hash);

        Ok(())
    }

    pub async fn is_avs(&self) -> Result<bool, Error> {
        let is_avs = self
            .inner
            .stake_registry_contract
            .operatorRegistered(self.address())
            .call()
            .await
            .map_err(|error| (ErrorKind::IsAvs, error))?
            ._0;

        Ok(is_avs)
    }

    pub async fn register_avs(&self) -> Result<(), Error> {
        let salt = [0u8; 32];
        let salt = FixedBytes::from_slice(&salt);
        let now = Utc::now().timestamp();
        let expiry: U256 = U256::from(now + 3600);
        let digest_hash = self
            .inner
            .avs_directory_contract
            .calculateOperatorAVSRegistrationDigestHash(
                self.address(),
                *self.inner.avs_contract.address(),
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
            .inner
            .stake_registry_contract
            .registerOperatorWithSignature(self.address(), operator_signature)
            .gas(300000)
            .gas_price(20000000000)
            .send()
            .await
            .map_err(|error| (ErrorKind::RegisterOnAvs, error))?
            .get_receipt()
            .await
            .map_err(|error| (ErrorKind::RegisterOnAvs, error))?;

        println!("{:?}", register_operator_with_signature.block_number);
        println!("{:?}", register_operator_with_signature.transaction_hash);

        Ok(())
    }

    pub async fn is_registered(
        &self,
        cluster_id: impl AsRef<str>,
        sequencer_address: Address,
    ) -> Result<bool, Error> {
        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterId, error))?;

        let sequencer_address_list = self
            .inner
            .ssal_contract
            .getSequencers(cluster_id)
            .call()
            .await
            .map_err(|error| (ErrorKind::IsRegistered, error))?
            ._0;

        for address in sequencer_address_list {
            if address == sequencer_address {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn initialize_cluster(
        &self,
        rollup_address: impl AsRef<str>,
        sequencer_rpc_url: impl AsRef<str>,
    ) -> Result<(), Error> {
        let rollup_address = Address::from_str(rollup_address.as_ref())
            .map_err(|error| (ErrorKind::ParseRollupAddress, error))?;
        let sequencer_rpc_url = sequencer_rpc_url.as_ref().to_owned();

        self.inner
            .seeder_client
            .register(self.address(), sequencer_rpc_url)
            .await?;

        let _transaction = self
            .inner
            .ssal_contract
            .initializeCluster(self.address(), rollup_address)
            .send()
            .await
            .map_err(|error| (ErrorKind::InitializeCluster, error))?;

        Ok(())
    }

    pub async fn register_sequencer(
        &self,
        cluster_id: impl AsRef<str>,
        sequencer_rpc_url: impl AsRef<str>,
    ) -> Result<(), Error> {
        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterId, error))?;
        let sequencer_rpc_url = sequencer_rpc_url.as_ref().to_owned();

        self.inner
            .seeder_client
            .register(self.address(), sequencer_rpc_url)
            .await?;

        let _transaction = self
            .inner
            .ssal_contract
            .registerSequencer(cluster_id, self.address())
            .send()
            .await
            .map_err(|error| (ErrorKind::RegisterSequencer, error))?;

        Ok(())
    }

    pub async fn deregister_sequencer(&self, cluster_id: impl AsRef<str>) -> Result<(), Error> {
        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterId, error))?;

        let _transaction = self
            .inner
            .ssal_contract
            .deregisterSequencer(cluster_id, self.address())
            .send()
            .await
            .map_err(|error| (ErrorKind::DeregisterSequencer, error))?;

        Ok(())
    }

    pub async fn register_block_commitment(
        &self,
        block_commitment: impl AsRef<[u8]>,
        block_number: u64,
        rollup_id: u32,
        cluster_id: impl AsRef<str>,
    ) -> Result<(), Error> {
        let block_commitment = Bytes::from_iter(block_commitment.as_ref());

        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterId, error))?;

        let _transaction = self
            .inner
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
        block_commitment: Bytes,
    ) -> Result<(), Error> {
        let message_k256 = keccak256(block_commitment);
        let message_hash = eip191_hash_message(message_k256);
        let signature = self
            .signer()
            .sign_hash(&message_hash)
            .await
            .map_err(|error| (ErrorKind::SignTask, error))?;

        let _transaction = self
            .inner
            .avs_contract
            .respondToTask(task, task_index, signature.as_bytes().into())
            .send()
            .await
            .map_err(|error| (ErrorKind::RespondToTask, error))?;

        Ok(())
    }

    pub async fn get_sequencer_list(
        &self,
        cluster_id: impl AsRef<str>,
    ) -> Result<Vec<(Address, Option<String>)>, Error> {
        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterId, error))?;

        let sequencer_address_list: [Address; 30] = self
            .inner
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
            .inner
            .seeder_client
            .get_sequencer_rpc_urls(sequencer_address_list.clone())
            .await?;

        let sequencer_list = zip(sequencer_address_list, sequencer_rpc_url_list).collect();

        Ok(sequencer_list)
    }
}
