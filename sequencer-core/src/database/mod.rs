pub mod client;
pub mod lock;

use std::fmt::Debug;

use serde::{de::DeserializeOwned, ser::Serialize};

use crate::error::Error;

pub trait Data: Clone + Debug + DeserializeOwned + Serialize {
    fn id() -> &'static str
    where
        Self: Sized;

    fn to_vec(&self) -> Result<Vec<u8>, Error>;

    fn from_slice(vec: &[u8]) -> Result<Self, Error>;
}
