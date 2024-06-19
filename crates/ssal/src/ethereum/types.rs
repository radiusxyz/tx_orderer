pub use alloy::{
    primitives::Address,
    rpc::types::{Block, Log},
    sol,
    sol_types::SolEvent,
};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Ssal,
    "src/ethereum/contract/Ssal.json"
);

pub enum SsalEventType {
    NewBlock(Block),
    InitializeCluster((Ssal::InitializeClusterEvent, Log)),
    BlockCommitment((Ssal::BlockCommitmentEvent, Log)),
}
