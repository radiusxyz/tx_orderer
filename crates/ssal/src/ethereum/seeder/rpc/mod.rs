mod deregister;
mod get_address_list;
mod register;
mod prelude {
    pub use json_rpc::RpcMethod;
    pub use serde::{Deserialize, Serialize};

    pub use crate::ethereum::types::*;
}

pub use deregister::Deregister;
pub use get_address_list::GetAddressList;
pub use register::Register;
