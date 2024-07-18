pub mod deregister;
pub mod get_sequencer_url_list;
pub mod register;
mod prelude {
    pub use std::sync::Arc;

    pub use database::database;
    pub use json_rpc::{types::*, RpcError};
    pub use ssal::avs::{seeder::rpc::*, types::*};

    pub use crate::util::health_check;
}
