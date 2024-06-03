pub mod external;
pub mod internal;
pub mod prelude {
    pub use async_trait::async_trait;
    pub use database::{database, Database, Lock};
    pub use json_rpc::{RpcError, RpcMethod};
    pub use serde::{Deserialize, Serialize};

    pub use crate::{error::Error, task::syncer::*, types::*};
}
