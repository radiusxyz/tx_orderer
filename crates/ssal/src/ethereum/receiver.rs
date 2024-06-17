use std::{future::Future, str::FromStr, sync::Arc};

use ethers::providers::{Middleware, Provider, StreamExt, Ws};

use crate::ethereum::{types::*, Error, ErrorKind};

pub struct SsalListener {
    client: Arc<Provider<Ws>>,
    contract: Ssal<Provider<Ws>>,
}

unsafe impl Send for SsalListener {}

unsafe impl Sync for SsalListener {}

impl Clone for SsalListener {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            contract: self.contract.clone(),
        }
    }
}

impl SsalListener {
    pub async fn init(
        ssal_rpc_address: impl AsRef<str>,
        contract_address: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let endpoint = format!("ws://{}", ssal_rpc_address.as_ref());
        let provider = Provider::<Ws>::connect(endpoint)
            .await
            .map_err(|error| (ErrorKind::InitializeEventListener, error))?;
        let client = Arc::new(provider);

        let contract_address = H160::from_str(contract_address.as_ref())
            .map_err(|error| Error::boxed(ErrorKind::ParseContractAddress, error))?;
        let contract = Ssal::new(contract_address, client.clone());

        Ok(Self { client, contract })
    }

    pub async fn block_subscriber<CB, CTX, F, R>(
        &self,
        callback: CB,
        context: CTX,
    ) -> Result<(), Error>
    where
        CB: Fn(Block<H256>, CTX) -> F + Send,
        CTX: Clone + Send + Sync,
        F: Future<Output = R> + Send,
        R: Send + 'static,
    {
        let mut block_stream = self
            .client
            .subscribe_blocks()
            .await
            .map_err(|error| (ErrorKind::BlockStream, error))?;

        while let Some(block) = block_stream.next().await {
            callback(block, context.clone()).await;
        }

        Err(Error::custom(
            ErrorKind::WebsocketDisconnected,
            "Block stream returned None",
        ))
    }

    pub async fn event_subscriber<CB, CTX, F, R>(
        &self,
        callback: CB,
        context: CTX,
    ) -> Result<(), Error>
    where
        CB: Fn(SsalEvents, CTX) -> F + Send,
        CTX: Clone + Send + Sync,
        F: Future<Output = R> + Send,
        R: Send + 'static,
    {
        let latest_block_number = self
            .client
            .get_block(BlockNumber::Latest)
            .await
            .map_err(|error| (ErrorKind::GetBlockNumber, error))?
            .ok_or(Error::custom(
                ErrorKind::GetBlockNumber,
                "Block number returned None",
            ))?
            .number
            .ok_or(Error::custom(
                ErrorKind::GetBlockNumber,
                "Cannot get the block number of a pending block",
            ))?;

        let events = self.contract.events().from_block(latest_block_number);
        let mut event_stream = events
            .subscribe()
            .await
            .map_err(|error| Error::boxed(ErrorKind::EventStream, error))?;

        while let Some(Ok(event)) = event_stream.next().await {
            callback(event, context.clone()).await;
        }

        Err(Error::custom(
            ErrorKind::WebsocketDisconnected,
            "Event stream returned None",
        ))
    }
}
