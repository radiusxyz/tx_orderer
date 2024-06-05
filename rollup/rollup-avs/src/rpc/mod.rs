mod build_block;
mod get_block;
mod prelude {
    pub use json_rpc::{RpcClient, RpcMethod};
    pub use serde::{Deserialize, Serialize};
    pub use ssal::ethereum::types::*;

    pub use crate::{
        error::{Error, ErrorKind},
        types::*,
    };
}

pub use build_block::BuildBlock;
pub use get_block::GetBlock;
