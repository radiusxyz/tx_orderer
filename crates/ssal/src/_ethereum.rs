use std::{io, str::FromStr, sync::Arc};

use ethers::{
    contract::{abigen, Contract},
    middleware::{Middleware, SignerMiddleware},
    providers::{Http, Provider, StreamExt, Ws},
    signers::{LocalWallet, Signer},
    types::{BlockNumber, Chain, H160, U64},
};

pub struct Client {}

// pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// abigen!(Ssal, "ssal_contract/artifacts/contracts/Ssal.sol/Ssal.json");

// pub struct Client {
//     signer: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
//     contract_address: H160,
// }

// impl Clone for Client {
//     fn clone(&self) -> Self {
//         Self {
//             signer: self.signer.clone(),
//             contract_address: self.contract_address.clone(),
//         }
//     }
// }

// impl Client {
//     pub fn new(
//         ssal_endpoint: impl AsRef<str>,
//         wallet_private_key: impl AsRef<str>,
//         contract_address: impl AsRef<str>,
//     ) -> Result<Self> {
//         let url = format!("http://{}", ssal_endpoint.as_ref());
//         let provider = Provider::<Http>::try_from(&url)?;
//         let wallet = wallet_private_key
//             .as_ref()
//             .parse::<LocalWallet>()?
//             .with_chain_id(Chain::AnvilHardhat);
//         let signer = Arc::new(SignerMiddleware::new(provider, wallet));
//         let contract_address = H160::from_str(contract_address.as_ref())?;
//         Ok(Self {
//             signer,
//             contract_address,
//         })
//     }

//     pub async fn get_block_number(&self) -> Result<U64> {
//         let block_number = self.signer.clone().get_block_number().await?;
//         Ok(block_number)
//     }

//     pub async fn get_sequencer_list(&self, cluster_id: [u8; 32], block_number: U64) -> Result<()> {
//         let contract = Ssal::new(self.contract_address, self.signer.clone());
//         let sequencer_list: [H160; 30] = contract
//             .get_sequencers(cluster_id)
//             .block(block_number)
//             .call()
//             .await?;
//         println!("{:?}", sequencer_list);
//         Ok(())
//     }
// }
