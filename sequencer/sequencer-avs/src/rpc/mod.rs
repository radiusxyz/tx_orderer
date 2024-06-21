pub mod external;
pub mod internal;
pub mod prelude {
    pub use std::sync::Arc;

    pub use database::{database, Lock};
    pub use json_rpc::{types::*, RpcClient, RpcError};
    pub use serde::{Deserialize, Serialize};

    pub use crate::{error::Error, state::AppState, task::*, types::*};
}
