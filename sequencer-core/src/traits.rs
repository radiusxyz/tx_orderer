use std::fmt::Debug;

use serde::{de::DeserializeOwned, ser::Serialize};

pub trait Data: Clone + Debug + DeserializeOwned + Serialize {
    fn id() -> &'static str;
}

pub trait Event: Data {
    fn id(&self) -> &'static str;
}
