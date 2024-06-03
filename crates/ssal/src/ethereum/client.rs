use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use ethers::{
    contract::abigen,
    providers::{Http, Middleware, Provider},
    types::H160,
};

use crate::ethereum::{types::*, Error, ErrorKind};

abigen!(Ssal, "src/ethereum/contract/Ssal.json");

pub struct SsalClient {
    provider: Arc<Provider<Http>>,
    contract_address: H160,
    cluster_id: [u8; 32],
    block_number: Arc<AtomicU64>,
}

unsafe impl Send for SsalClient {}

unsafe impl Sync for SsalClient {}

impl Clone for SsalClient {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            contract_address: self.contract_address.clone(),
            cluster_id: self.cluster_id.clone(),
            block_number: self.block_number.clone(),
        }
    }
}

impl SsalClient {
    pub fn new(
        ssal_address: impl AsRef<str>,
        contract_address: impl AsRef<str>,
        cluster_id: [u8; 32],
    ) -> Result<Self, Error> {
        let rpc_endpoint = format!("http://{}", ssal_address.as_ref());
        let provider = Provider::<Http>::try_from(rpc_endpoint)
            .map_err(|error| (ErrorKind::BuildSsalClient, error))?;
        let contract_address = H160::from_str(contract_address.as_ref())
            .map_err(|error| (ErrorKind::ParseContractAddress, error))?;

        Ok(Self {
            provider: Arc::new(provider),
            contract_address,
            cluster_id,
            block_number: Arc::new(AtomicU64::default()),
        })
    }

    fn block_number(&self) -> u64 {
        self.block_number.load(Ordering::SeqCst)
    }

    fn update_block_number(&self, block_number: u64) {
        self.block_number.store(block_number, Ordering::SeqCst)
    }

    async fn get_latest_block_number(&self) -> Result<u64, Error> {
        let block_number = self
            .provider
            .get_block_number()
            .await
            .map_err(|error| (ErrorKind::GetBlockNumber, error))?
            .as_u64();
        Ok(block_number)
    }

    pub async fn get_sequencer_list(&self) -> Result<Option<Vec<PublicKey>>, Error> {
        let latest_block_number = self.get_latest_block_number().await?;
        if self.block_number() != latest_block_number {
            let contract = Ssal::new(self.contract_address, self.provider.clone());
            let sequencer_list: [H160; 30] = contract
                .get_sequencers(self.cluster_id)
                .block(latest_block_number)
                .call()
                .await
                .map_err(|error| (ErrorKind::GetSequencerList, error))?;

            let sequencer_list: Vec<PublicKey> = sequencer_list
                .into_iter()
                .filter(|public_key| !public_key.is_zero())
                .map(PublicKey::from)
                .collect();

            self.update_block_number(latest_block_number);
            Ok(Some(sequencer_list))
        } else {
            Ok(None)
        }
    }
}
