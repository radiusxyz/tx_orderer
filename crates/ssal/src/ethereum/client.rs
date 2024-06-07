use std::{str::FromStr, sync::Arc};

use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet},
    types::{Chain, H160},
};

use crate::ethereum::{seeder::SeederClient, types::*, Error, ErrorKind};

abigen!(Ssal, "src/ethereum/contract/Ssal.json");

pub struct SsalClient {
    signer: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    contract: Ssal<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
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
    pub fn new(
        ssal_rpc_address: impl AsRef<str>,
        ssal_private_key: impl AsRef<str>,
        contract_address: impl AsRef<str>,
        cluster_id: [u8; 32],
        seeder_rpc_address: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let provider = Provider::<Http>::try_from(ssal_rpc_address.as_ref())
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

    pub async fn get_latest_block_number(&self) -> Result<u64, Error> {
        let block_number = self
            .signer
            .get_block_number()
            .await
            .map_err(|error| Error::boxed(ErrorKind::GetBlockNumber, error))?
            .as_u64();
        Ok(block_number)
    }

    pub async fn initialize_cluster(&self) -> Result<(), Error> {
        // The seeder must respond in order to minimize the hassle.
        // self.seeder_client
        //     .register(self.signer.address().into(), sequencer_rpc_address.into())
        //     .await?;
        // self.contract.initialize_cluster(sequencer, rollup);
        Ok(())
    }

    pub async fn register_sequencer(&self, sequencer_rpc_address: String) -> Result<(), Error> {
        // The seeder must respond in order to minimize the hassle.
        self.seeder_client
            .register(self.signer.address().into(), sequencer_rpc_address.into())
            .await?;
        self.contract
            .register_sequencer(self.cluster_id, self.signer.address())
            .send()
            .await
            .map_err(|error| Error::boxed(ErrorKind::Register, error))?;
        Ok(())
    }

    pub async fn deregister_sequencer(&self) -> Result<(), Error> {
        self.contract
            .deregister_sequencer(self.cluster_id, self.signer.address())
            .send()
            .await
            .map_err(|error| Error::boxed(ErrorKind::Deregister, error))?;

        // Deregistering does not depend on deleting the sequencer RPC address from seeder.
        // Therefore, it is safe to ignore any errors.
        let _ = self
            .seeder_client
            .deregister(self.signer.address().into())
            .await;
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
}

// pub struct SsalClient {
//     provider: Arc<Provider<Http>>,
//     contract_address: H160,
//     cluster_id: [u8; 32],
//     block_number: Arc<AtomicU64>,
// }

// unsafe impl Send for SsalClient {}

// unsafe impl Sync for SsalClient {}

// impl Clone for SsalClient {
//     fn clone(&self) -> Self {
//         Self {
//             provider: self.provider.clone(),
//             contract_address: self.contract_address.clone(),
//             cluster_id: self.cluster_id.clone(),
//             block_number: self.block_number.clone(),
//         }
//     }
// }

// impl SsalClient {
//     pub fn new(
//         ssal_address: impl AsRef<str>,
//         contract_address: impl AsRef<str>,
//         cluster_id: [u8; 32],
//     ) -> Result<Self, Error> {
//         let rpc_endpoint = format!("http://{}", ssal_address.as_ref());
//         let provider = Provider::<Http>::try_from(rpc_endpoint)
//             .map_err(|error| Error::boxed(ErrorKind::BuildSsalClient, error))?;
//         let contract_address = H160::from_str(contract_address.as_ref())
//             .map_err(|error| Error::boxed(ErrorKind::ParseContractAddress, error))?;

//         Ok(Self {
//             provider: Arc::new(provider),
//             contract_address,
//             cluster_id,
//             block_number: Arc::new(AtomicU64::default()),
//         })
//     }

//     fn block_number(&self) -> u64 {
//         self.block_number.load(Ordering::SeqCst)
//     }

//     fn update_block_number(&self, block_number: u64) {
//         self.block_number.store(block_number, Ordering::SeqCst)
//     }

//     async fn get_latest_block_number(&self) -> Result<u64, Error> {
//         let block_number = self
//             .provider
//             .get_block_number()
//             .await
//             .map_err(|error| (ErrorKind::GetBlockNumber, error))?
//             .as_u64();
//         Ok(block_number)
//     }

//     pub async fn get_sequencer_list(&self) -> Result<Option<(u64, Vec<PublicKey>)>, Error> {
//         let latest_block_number = self.get_latest_block_number().await?;
//         if self.block_number() != latest_block_number {
//             let contract = Ssal::new(self.contract_address, self.provider.clone());
//             let sequencer_list: [H160; 30] = contract
//                 .get_sequencers(self.cluster_id)
//                 .block(latest_block_number)
//                 .call()
//                 .await
//                 .map_err(|error| (ErrorKind::GetSequencerList, error))?;

//             let sequencer_list: Vec<PublicKey> = sequencer_list
//                 .into_iter()
//                 .filter(|public_key| !public_key.is_zero())
//                 .map(PublicKey::from)
//                 .collect();

//             self.update_block_number(latest_block_number);
//             Ok(Some((latest_block_number, sequencer_list)))
//         } else {
//             Ok(None)
//         }
//     }
// }
