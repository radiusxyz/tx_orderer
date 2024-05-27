use std::sync::Arc;

use database::client::Database;
use ethers::{
    contract::{abigen, Contract},
    core::k256::ecdsa::SigningKey,
    middleware::{Middleware, SignerMiddleware},
    providers::{Http, Provider, StreamExt, Ws},
    signers::{Signer, Wallet},
    types::Chain,
};
use primitives::error::Error;

pub struct EthereumClient {
    sender: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
}

impl EthereumClient {
    pub fn init(
        ethereum_rpc_endpoint: impl AsRef<str>,
        wallet_private_key: impl AsRef<str>,
        database: Database,
    ) -> Result<Self, Error> {
        let sender =
            Self::init_sender(ethereum_rpc_endpoint.as_ref(), wallet_private_key.as_ref())?;
        Ok(Self { sender })
    }

    fn init_sender(
        ethereum_rpc_endpoint: impl AsRef<str>,
        wallet_private_key: impl AsRef<str>,
    ) -> Result<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, Error> {
        let rpc_endpoint = format!("http://{}", ethereum_rpc_endpoint.as_ref());
        let provider = Provider::<Http>::try_from(rpc_endpoint).map_err(Error::new)?;
        let wallet = wallet_private_key
            .as_ref()
            .parse::<Wallet<SigningKey>>()
            .map_err(Error::new)?
            .with_chain_id(Chain::AnvilHardhat);
        let sender = Arc::new(SignerMiddleware::new(provider, wallet));
        Ok(sender)
    }

    async fn init_listener(ethereum_rpc_endpoint: impl AsRef<str>) -> Result<(), Error> {
        let rpc_endpoint = format!("ws://{}", ethereum_rpc_endpoint.as_ref());
        let listener = Arc::new(
            Provider::<Ws>::connect(rpc_endpoint)
                .await
                .map_err(Error::new)?,
        );
        Ok(())
    }
}

impl super::Client for EthereumClient {
    async fn initialize_cluster(&self) -> Result<(), Box<dyn std::error::Error>> {}

    async fn register_sequencer(&self) -> Result<(), Box<dyn std::error::Error>> {}

    async fn deregister_sequencer(&self) -> Result<(), Box<dyn std::error::Error>> {}
}
