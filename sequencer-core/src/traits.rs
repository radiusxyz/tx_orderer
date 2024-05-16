use std::fmt::Debug;

use serde::{de::DeserializeOwned, ser::Serialize};

pub trait Data: Clone + Debug + DeserializeOwned + Serialize {}

pub trait Event: Send + 'static {
    fn id(&self) -> &'static str
    where
        Self: Sized;
}
