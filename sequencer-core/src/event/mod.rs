pub mod manager;
pub mod subscription;

use std::fmt::Debug;

use serde::{de::DeserializeOwned, ser::Serialize};

pub trait Event: Clone + Debug + DeserializeOwned + Serialize {
    fn id() -> &'static str;

    fn to_vec(&self) -> Vec<u8>;

    fn from_vec(vec: Vec<u8>) -> Self;
}
