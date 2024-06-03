mod client;
mod error;
mod lock;
#[cfg(feature = "singleton")]
mod singleton;

pub use client::Database;
pub use error::{Error, ErrorKind};
pub use lock::Lock;
#[cfg(feature = "singleton")]
pub use singleton::*;
