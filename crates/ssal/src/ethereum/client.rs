use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use ethers::{
    contract::abigen,
    providers::{Http, Provider},
    types::H160,
};
use primitives::{error::Error, types::SequencerAddress};

use crate::ethereum::types::*;

abigen!(Ssal, "ssal_contract/artifacts/contracts/Ssal.sol/Ssal.json");

pub struct SsalClient {
    provider: Arc<Provider<Http>>,
    contract_address: H160,
    cluster_id: [u8; 32],
    ssal_block_height: Arc<AtomicU64>,
}

unsafe impl Send for SsalClient {}

unsafe impl Sync for SsalClient {}

impl Clone for SsalClient {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            contract_address: self.contract_address.clone(),
            cluster_id: self.cluster_id.clone(),
            ssal_block_height: self.ssal_block_height.clone(),
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
        let provider = Provider::<Http>::try_from(rpc_endpoint).map_err(Error::new)?;
        let contract_address = H160::from_str(contract_address.as_ref()).map_err(Error::new)?;

        Ok(Self {
            provider: Arc::new(provider),
            contract_address,
            cluster_id,
            ssal_block_height: Arc::new(AtomicU64::default()),
        })
    }

    async fn get_sequencer_list(&self) -> Result<Vec<PublicKey>, Error> {
        let contract = Ssal::new(self.contract_address, self.provider.clone());
        let sequencer_list: [H160; 30] = contract
            .get_sequencers(self.cluster_id)
            .block()
            .call()
            .await
            .map_err(Error::new)?;
        let sequencer_list: Vec<Option<SequencerAddress>> = sequencer_list
            .iter()
            .map(|sequencer_public_key| {
                if sequencer_public_key.is_zero() {
                    None
                } else {
                    Some(SequencerAddress::from(sequencer_public_key))
                }
            })
            .collect();
        Ok(sequencer_list)
    }
}
