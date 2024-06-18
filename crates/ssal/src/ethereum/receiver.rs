use std::{
    future::Future,
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
};

use ethers::{
    contract::{stream::EventStream, ContractError, LogMeta},
    providers::{Middleware, Provider, StreamExt, SubscriptionStream, Ws},
};
use futures::{stream::select_all, Stream};
use pin_project::pin_project;

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

    pub async fn with_callback<CB, CTX, F>(&self, callback: CB, context: CTX) -> Result<(), Error>
    where
        CB: Fn(SsalEventType, CTX) -> F + Send,
        CTX: Clone + Send + Sync,
        F: Future<Output = ()> + Send,
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

        let block_stream: SsalEventStream = self
            .client
            .subscribe_blocks()
            .await
            .map_err(|error| (ErrorKind::BlockStream, error))?
            .into();

        let events = self.contract.events().from_block(latest_block_number);
        let event_stream: SsalEventStream = events
            .subscribe_with_meta()
            .await
            .map_err(|error| Error::boxed(ErrorKind::EventStream, error))?
            .into();

        let mut ssal_event_stream = select_all(vec![block_stream, event_stream]);
        while let Some(event) = ssal_event_stream.next().await {
            callback(event, context.clone()).await;
        }

        Ok(())
    }
}

#[pin_project(project = StreamInner)]
enum SsalEventStream<'stream> {
    Block(SubscriptionStream<'stream, Ws, Block<H256>>),
    Event(
        EventStream<
            'stream,
            SubscriptionStream<'stream, Ws, Log>,
            (SsalEvents, LogMeta),
            ContractError<Provider<Ws>>,
        >,
    ),
}

impl<'stream> From<SubscriptionStream<'stream, Ws, Block<H256>>> for SsalEventStream<'stream> {
    fn from(value: SubscriptionStream<'stream, Ws, Block<H256>>) -> Self {
        Self::Block(value)
    }
}

impl<'stream>
    From<
        EventStream<
            'stream,
            SubscriptionStream<'stream, Ws, Log>,
            (SsalEvents, LogMeta),
            ContractError<Provider<Ws>>,
        >,
    > for SsalEventStream<'stream>
{
    fn from(
        value: EventStream<
            'stream,
            SubscriptionStream<'stream, Ws, Log>,
            (SsalEvents, LogMeta),
            ContractError<Provider<Ws>>,
        >,
    ) -> Self {
        Self::Event(value)
    }
}

impl<'stream> Stream for SsalEventStream<'stream> {
    type Item = SsalEventType;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            StreamInner::Block(stream) => stream
                .poll_next_unpin(cx)
                .map(|output| output.map(SsalEventType::from)),
            StreamInner::Event(stream) => stream.poll_next_unpin(cx).map(|output| {
                output.map(|output| match output {
                    Ok(response) => SsalEventType::from(response),
                    Err(error) => SsalEventType::from(error),
                })
            }),
        }
    }
}
