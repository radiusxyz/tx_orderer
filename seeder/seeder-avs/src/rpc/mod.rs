pub mod deregister;
pub mod get_address_list;
pub mod register;
mod prelude {
    pub use std::sync::Arc;

    pub use database::database;
    pub use json_rpc::{types::*, RpcError};
    pub use ssal::ethereum::{seeder::rpc::*, types::*};

    pub use crate::util::health_check;
}
