mod deregister;
mod get_address_list;
mod register;
mod prelude {
    pub use async_trait::async_trait;
    pub use database::database;
    pub use json_rpc::{RpcError, RpcMethod};
    pub use serde::{Deserialize, Serialize};
    pub use ssal::ethereum::types::*;

    pub use crate::{error::Error, util::health_check};
}

pub use deregister::Deregister;
pub use get_address_list::GetAddressList;
pub use register::Register;
