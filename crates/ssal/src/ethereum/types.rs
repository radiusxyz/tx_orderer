pub use ethers::types::*;
use ethers::{
    contract::{ContractError, LogMeta},
    providers::{Provider, Ws},
};

ethers::contract::abigen!(Ssal, "src/ethereum/contract/Ssal.json");

pub enum SsalEventType {
    NewBlock(Block<H256>),
    ContractError(ContractError<Provider<Ws>>),
    InitializeCluster((InitializeClusterEventFilter, LogMeta)),
    BlockCommitmentEvent((BlockCommitmentEventFilter, LogMeta)),
}

impl From<Block<H256>> for SsalEventType {
    fn from(value: Block<H256>) -> Self {
        Self::NewBlock(value)
    }
}

impl From<ContractError<Provider<Ws>>> for SsalEventType {
    fn from(value: ContractError<Provider<Ws>>) -> Self {
        Self::ContractError(value)
    }
}

impl From<(SsalEvents, LogMeta)> for SsalEventType {
    fn from(value: (SsalEvents, LogMeta)) -> Self {
        match value.0 {
            SsalEvents::InitializeClusterEventFilter(event) => {
                Self::InitializeCluster((event, value.1))
            }
            SsalEvents::BlockCommitmentEventFilter(event) => {
                Self::BlockCommitmentEvent((event, value.1))
            }
        }
    }
}
