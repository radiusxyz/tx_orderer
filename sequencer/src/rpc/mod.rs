pub mod prelude {
    pub use std::{fmt::Debug, sync::Arc};

    pub use database::{client::Database, lock::Lock};
    pub use json_rpc::method::RpcMethod;
    pub use primitives::{
        async_trait::async_trait,
        error::Error,
        serde::{Deserialize, Serialize},
    };
}

mod send_transaction;
pub use self::send_transaction::SendEncryptedTransaction;
