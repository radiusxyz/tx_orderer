pub mod external;
pub mod internal;
pub mod util;
pub mod prelude {
    pub use std::sync::Arc;

    pub use database::Database;
    pub use json_rpc::{types::*, RpcClient, RpcError};
    pub use serde::{Deserialize, Serialize};

    pub use crate::{error::Error, state::AppState, task::*, types::*};
}
