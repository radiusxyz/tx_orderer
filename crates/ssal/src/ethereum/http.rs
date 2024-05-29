use std::{str::FromStr, sync::Arc};

use ethers::prelude::*;
use primitives::error::Error;

// pub struct Client {}

// impl Client {
//     pub fn new(
//         ssal_endpoint: impl AsRef<str>,
//         wallet_private_key: Option<impl AsRef<str>>,
//         contract_address: impl AsRef<str>,
//     ) -> Result<Self, Error> {
//         let url = format!("http://{}", ssal_endpoint.as_ref());
//         let provider = Provider::<Http>::try_from(&url).map_err(Error::new)?;
//         let wallet = wallet_private_key
//             .as_ref()
//             .parse::<LocalWallet>()
//             .map_err(Error::new)?
//             .with_chain_id(Chain::AnvilHardhat);
//         let signer = Arc::new(SignerMiddleware::new(provider, wallet));
//         let contract_address = H160::from_str(contract_address.as_ref()).map_err(Error::new)?;
//         Ok(Self {
//             signer,
//             contract_address,
//         })
//     }
// }
