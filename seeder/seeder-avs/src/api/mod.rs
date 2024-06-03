mod address_list;
mod deregister;
mod register;
mod prelude {
    pub use std::net::SocketAddr;

    pub use axum::{
        extract::{ConnectInfo, Query, State},
        http::StatusCode,
        response::IntoResponse,
        Json,
    };
    pub use database::Database;
    pub use serde::{Deserialize, Serialize};
    pub use ssal::ethereum::{PublicKey, Signature};

    pub use crate::error::Error;
}

pub use address_list::AddressList;
pub use deregister::Deregister;
pub use register::Register;
