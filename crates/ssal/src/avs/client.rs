use std::{iter::zip, path::Path, str::FromStr};

use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::FixedBytes,
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, ProviderBuilder, RootProvider,
    },
    signers::local::LocalSigner,
    transports::http::{Client, Http},
};

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

pub struct SsalClient {
    provider: EthereumProvider,
    ssal_contract: SsalContract,
    // TODO: Uncomment after EigenLayer integration
    // avs_contract: AvsContract,
    seeder_client: SeederClient,
}

unsafe impl Send for SsalClient {}

unsafe impl Sync for SsalClient {}

impl Clone for SsalClient {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            ssal_contract: self.ssal_contract.clone(),
            seeder_client: self.seeder_client.clone(),
        }
    }
}

impl SsalClient {
    pub fn new(
        ethereum_rpc_url: impl AsRef<str>,
        keystore_path: impl AsRef<Path>,
        keystore_password: impl AsRef<[u8]>,
        ssal_contract_address: impl AsRef<str>,
        // TODO: Uncomment after EigenLayer integration
        // avs_contract_address: impl AsRef<str>,
        seeder_rpc_url: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let url = ethereum_rpc_url
            .as_ref()
            .parse()
            .map_err(|error| Error::boxed(ErrorKind::ParseRpcUrl, error))?;

        let signer = LocalSigner::decrypt_keystore(keystore_path, keystore_password)
            .map_err(|error| (ErrorKind::Keystore, error))?;
        let wallet = EthereumWallet::from(signer);

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);

        let ssal_contract_address = Address::from_str(ssal_contract_address.as_ref())
            .map_err(|error| (ErrorKind::ParseSsalContractAddress, error))?;
        let ssal_contract = Ssal::SsalInstance::new(ssal_contract_address, provider.clone());

        // TODO: Uncomment after EigenLayer integration
        // let avs_contract_address = Address::from_str(avs_contract_address.as_ref())
        //     .map_err(|error| (ErrorKind::ParseContractAddress, error))?;

        let seeder_client = SeederClient::new(seeder_rpc_url)?;

        Ok(Self {
            provider,
            ssal_contract,
            seeder_client,
        })
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
            .map_err(|error| (ErrorKind::ParseClusterID, error))?;
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
            .map_err(|error| (ErrorKind::ParseClusterID, error))?;
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
        cluster_id: impl AsRef<str>,
        block_number: u64,
        block_commitment: String,
    ) -> Result<(), Error> {
        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterID, error))?;

        let _transaction = self
            .ssal_contract
            .registerBlockCommitment(cluster_id, block_number, block_commitment)
            .send()
            .await
            .map_err(|error| (ErrorKind::RegisterBlockCommitment, error))?;

        Ok(())
    }

    pub async fn get_sequencer_list(
        &self,
        cluster_id: impl AsRef<str>,
    ) -> Result<Vec<(Address, Option<String>)>, Error> {
        let cluster_id = FixedBytes::from_str(cluster_id.as_ref())
            .map_err(|error| (ErrorKind::ParseClusterID, error))?;

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
