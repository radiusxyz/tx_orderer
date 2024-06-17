use std::{future::Future, str::FromStr, sync::Arc};

use ethers::providers::{Middleware, Provider, StreamExt, Ws};

use crate::ethereum::{types::*, Error, ErrorKind};

pub struct SsalListener {
    client: Arc<Provider<Ws>>,
    contract: Ssal<Provider<Ws>>,
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
        CB: Fn(Block<H256>, CTX) -> F,
        CTX: Clone + Send + Sync,
        F: Future<Output = ()>,
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
        CB: Fn(SsalEvents, CTX) -> F,
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

// #[pin_project(project = StreamInner)]
// pub enum SsalEventStream<'a> {
//     BlockStream(SubscriptionStream<'a, Ws, Block<H256>>),
//     EventStream(EventStreamMeta<'a, Ws, (SsalEvents, LogMeta), Log>),
// }

// impl<'a> From<SubscriptionStream<'a, Ws, Block<H256>>> for SsalEventStream<'a> {
//     fn from(value: SubscriptionStream<'a, Ws, Block<H256>>) -> Self {
//         Self::BlockStream(value)
//     }
// }

// impl<'a> From<EventStreamMeta<'a, Ws, (SsalEvents, LogMeta), Log>> for SsalEventStream<'a> {
//     fn from(value: EventStreamMeta<'a, Ws, (SsalEvents, LogMeta), Log>) -> Self {
//         Self::EventStream(value)
//     }
// }

// impl<'a> Stream for SsalEventStream<'a> {
//     type Item = SsalEventType;

//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         match self.project() {
//             StreamInner::BlockStream(block_stream) => block_stream
//                 .poll_next_unpin(cx)
//                 .map(|block| Some(SsalEventType)),
//             StreamInner::EventStream(event_stream) => event_stream
//                 .poll_next_unpin(cx) // Resolve
//                 .map_ok(|(event, log), error| Some(SsalEventType)),
//         }
//     }
// }

// pub struct SsalEventType;
