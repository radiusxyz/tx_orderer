use std::{
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    rpc::types::{Block, Log},
    sol_types,
};
use futures::{stream::select_all, Future, Stream, StreamExt, TryStreamExt};
use pin_project::pin_project;

use super::types::{
    Avs::{self, NewTaskCreated},
    Ssal::{self, InitializeClusterEvent},
};
use crate::avs::{types::SsalEventType, Error, ErrorKind};

pub struct SsalEventListener {
    provider: RootProvider<PubSubFrontend>,
    ssal_contract_address: Address,
    avs_contract_address: Address,
}

impl SsalEventListener {
    pub async fn connect(
        ethereum_websocket_url: impl AsRef<str>,
        ssal_contract_address: impl AsRef<str>,
        avs_contract_address: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let websocket = WsConnect::new(ethereum_websocket_url.as_ref());
        let provider = ProviderBuilder::new()
            .on_ws(websocket)
            .await
            .map_err(|error| (ErrorKind::ConnectEventListener, error))?;

        let ssal_contract_address = Address::from_str(ssal_contract_address.as_ref())
            .map_err(|error| Error::boxed(ErrorKind::ParseSsalContractAddress, error))?;

        let avs_contract_address = Address::from_str(avs_contract_address.as_ref())
            .map_err(|error| Error::boxed(ErrorKind::ParseAvsContractAddress, error))?;

        Ok(Self {
            provider,
            ssal_contract_address,
            avs_contract_address,
        })
    }

    async fn push_block_stream(&self, stream: &mut Vec<EventStream>) -> Result<(), Error> {
        let block_stream: EventStream = self
            .provider
            .subscribe_blocks()
            .await
            .map_err(|error| (ErrorKind::BlockStream, error))?
            .into_stream()
            .boxed()
            .into();
        stream.push(block_stream);

        Ok(())
    }

    async fn push_ssal_event_stream(&self, stream: &mut Vec<EventStream>) -> Result<(), Error> {
        let ssal_contract =
            Ssal::SsalInstance::new(self.ssal_contract_address, self.provider.clone());

        let initialize_cluster_event_stream: EventStream = ssal_contract
            .InitializeClusterEvent_filter()
            .subscribe()
            .await
            .map_err(|error| (ErrorKind::InitializeClusterEventStream, error))?
            .into_stream()
            .boxed()
            .into();
        stream.push(initialize_cluster_event_stream);

        Ok(())
    }

    async fn push_avs_event_stream(&self, stream: &mut Vec<EventStream>) -> Result<(), Error> {
        let avs_contract = Avs::AvsInstance::new(self.avs_contract_address, self.provider.clone());

        let block_commitment_event_stream: EventStream = avs_contract
            .NewTaskCreated_filter()
            .subscribe()
            .await
            .map_err(|error| (ErrorKind::BlockCommitmentEventStream, error))?
            .into_stream()
            .boxed()
            .into();
        stream.push(block_commitment_event_stream);

        Ok(())
    }

    pub async fn init<CB, CTX, F>(self, callback: CB, context: CTX) -> Result<(), Error>
    where
        CB: Fn(SsalEventType, CTX) -> F,
        CTX: Clone + Send + Sync,
        F: Future<Output = ()>,
    {
        let mut stream_list = Vec::<EventStream>::new();
        self.push_block_stream(&mut stream_list).await?;
        self.push_ssal_event_stream(&mut stream_list).await?;
        self.push_avs_event_stream(&mut stream_list).await?;

        let mut event_stream = select_all(stream_list);
        while let Some(event) = event_stream.next().await {
            callback(event, context.clone()).await;
        }

        Err(Error::custom(
            ErrorKind::EventListener,
            "EventListener disconnected",
        ))
    }
}

type BlockStream = Pin<Box<dyn Stream<Item = Block> + Send>>;

type InitializeClusterEventStream =
    Pin<Box<dyn Stream<Item = Result<(InitializeClusterEvent, Log), sol_types::Error>> + Send>>;

type BlockCommitmentEventStream =
    Pin<Box<dyn Stream<Item = Result<(NewTaskCreated, Log), sol_types::Error>> + Send>>;

#[allow(unused)]
#[pin_project(project = StreamType)]
enum EventStream {
    Block(BlockStream),
    InitializeCluster(InitializeClusterEventStream),
    BlockCommitment(BlockCommitmentEventStream),
}

impl From<BlockStream> for EventStream {
    fn from(value: BlockStream) -> Self {
        Self::Block(value)
    }
}

impl From<InitializeClusterEventStream> for EventStream {
    fn from(value: InitializeClusterEventStream) -> Self {
        Self::InitializeCluster(value)
    }
}

impl From<BlockCommitmentEventStream> for EventStream {
    fn from(value: BlockCommitmentEventStream) -> Self {
        Self::BlockCommitment(value)
    }
}

impl Stream for EventStream {
    type Item = SsalEventType;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            StreamType::Block(stream) => stream.poll_next_unpin(cx).map(|event| match event {
                Some(block) => Some(SsalEventType::NewBlock(block)),
                None => None,
            }),
            StreamType::InitializeCluster(stream) => stream
                .try_poll_next_unpin(cx)
                .map_ok(|event| SsalEventType::InitializeCluster(event))
                .map(|event| match event {
                    Some(event) => event.ok(),
                    None => None,
                }),
            StreamType::BlockCommitment(stream) => stream
                .try_poll_next_unpin(cx)
                .map_ok(|event| SsalEventType::BlockCommitment(event))
                .map(|event| match event {
                    Some(event) => event.ok(),
                    None => None,
                }),
        }
    }
}
