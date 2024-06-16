use std::{future::Future, str::FromStr, sync::Arc};

use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Middleware, Provider, StreamExt, Ws},
    signers::{Signer, Wallet},
    types::{Chain, H160},
};

use crate::ethereum::{
    seeder::SeederClient,
    types::{internal::*, *},
    Error, ErrorKind,
};

pub struct SsalClient {
    signer: Arc<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>,
    contract: Ssal<SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>>,
    cluster_id: [u8; 32],
    seeder_client: Arc<SeederClient>,
}

unsafe impl Send for SsalClient {}

unsafe impl Sync for SeederClient {}

impl Clone for SsalClient {
    fn clone(&self) -> Self {
        Self {
            signer: self.signer.clone(),
            contract: self.contract.clone(),
            cluster_id: self.cluster_id,
            seeder_client: self.seeder_client.clone(),
        }
    }
}

impl SsalClient {
    pub async fn new(
        ssal_rpc_address: impl AsRef<str>,
        ssal_private_key: impl AsRef<str>,
        contract_address: impl AsRef<str>,
        cluster_id: [u8; 32],
        seeder_rpc_address: impl AsRef<str>,
    ) -> Result<Self, Error> {
        // let provider = Provider::<Http>::try_from(ssal_rpc_address.as_ref())
        let endpoint = format!("ws://{}", ssal_rpc_address.as_ref());
        let provider = Provider::<Ws>::connect(endpoint)
            .await
            .map_err(|error| Error::boxed(ErrorKind::BuildSsalClient, error))?;
        let wallet = ssal_private_key
            .as_ref()
            .parse::<Wallet<SigningKey>>()
            .map_err(|error| Error::boxed(ErrorKind::ParsePrivateKey, error))?
            .with_chain_id(Chain::AnvilHardhat);
        let signer = Arc::new(SignerMiddleware::new(provider, wallet));
        let contract_address = H160::from_str(contract_address.as_ref())
            .map_err(|error| Error::boxed(ErrorKind::ParseContractAddress, error))?;
        let contract = Ssal::new(contract_address, signer.clone());
        let seeder_client = Arc::new(SeederClient::new(seeder_rpc_address.as_ref())?);

        Ok(Self {
            signer,
            contract,
            cluster_id,
            seeder_client,
        })
    }

    pub fn public_key(&self) -> PublicKey {
        self.signer.address().into()
    }

    pub async fn get_latest_block_number(&self) -> Result<u64, Error> {
        let block_number = self
            .signer
            .get_block_number()
            .await
            .map_err(|error| Error::boxed(ErrorKind::GetBlockNumber, error))?
            .as_u64();
        Ok(block_number)
    }

    pub async fn initialize_cluster(
        &self,
        sequencer_rpc_address: impl AsRef<str>,
        rollup_public_key: impl AsRef<str>,
    ) -> Result<(), Error> {
        // The seeder must respond in order to minimize the hassle.
        self.seeder_client
            .register(
                self.signer.address().into(),
                sequencer_rpc_address.as_ref().into(),
            )
            .await?;
        self.contract
            .initialize_cluster(
                self.signer.address(),
                H160::from_str(rollup_public_key.as_ref())
                    .map_err(|error| Error::boxed(ErrorKind::ParsePublicKey, error))?,
            )
            .send()
            .await
            .map_err(|error| Error::boxed(ErrorKind::InitializeCluster, error))?;
        Ok(())
    }

    pub async fn register_sequencer(
        &self,
        sequencer_rpc_address: impl AsRef<str>,
    ) -> Result<(), Error> {
        // The seeder must respond in order to minimize the hassle.
        self.seeder_client
            .register(
                self.signer.address().into(),
                sequencer_rpc_address.as_ref().into(),
            )
            .await?;
        self.contract
            .register_sequencer(self.cluster_id, self.signer.address())
            .send()
            .await
            .map_err(|error| Error::boxed(ErrorKind::RegisterSequencer, error))?;
        Ok(())
    }

    pub async fn deregister_sequencer(&self) -> Result<(), Error> {
        self.contract
            .deregister_sequencer(self.cluster_id, self.signer.address())
            .send()
            .await
            .map_err(|error| Error::boxed(ErrorKind::DeregisterSequencer, error))?;
        Ok(())
    }

    pub async fn get_sequencer_list(
        &self,
        block_number: u64,
    ) -> Result<(Vec<PublicKey>, Vec<Option<RpcAddress>>), Error> {
        let sequencer_public_keys: [H160; 30] = self
            .contract
            .get_sequencers(self.cluster_id)
            .block(block_number)
            .call()
            .await
            .map_err(|error| Error::boxed(ErrorKind::GetSequencerList, error))?;
        let sequencer_list: Vec<PublicKey> = sequencer_public_keys
            .into_iter()
            .filter(|public_key| !public_key.is_zero())
            .map(PublicKey::from)
            .collect();

        let address_list = self
            .seeder_client
            .get_address_list(sequencer_list.clone())
            .await?;
        Ok((sequencer_list, address_list))
    }

    pub async fn block_number_subscriber<F, R, H>(&self, handler: H) -> Result<(), Error>
    where
        F: Future<Output = R>,
        R: Send + 'static,
        H: Fn(u64, SsalClient) -> F,
    {
        let mut block_stream = self
            .signer
            .subscribe_blocks()
            .await
            .map_err(|error| Error::boxed(ErrorKind::BlockSubscriber, error))?;
        while let Some(block) = block_stream.next().await {
            if let Some(block_number) = block.number {
                handler(block_number.as_u64(), self.clone()).await;
            }
        }
        Ok(())
    }
}
