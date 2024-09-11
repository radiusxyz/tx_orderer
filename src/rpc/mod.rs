pub mod cluster;
pub mod external;
pub mod internal;
pub(crate) mod prelude {
    pub use std::sync::Arc;

    pub use radius_sequencer_sdk::{
        json_rpc::{types::*, RpcClient, RpcError},
        signature::{ChainType, Signature},
    };
    pub use serde::{Deserialize, Serialize};

    pub use crate::{client::liveness, error::Error, state::AppState, types::*};

    pub fn serialize_to_bincode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(value)
    }
}
