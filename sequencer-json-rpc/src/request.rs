use std::fmt::Debug;

use sequencer_core::{
    caller,
    error::{Error, WrapError},
    serde::Serialize,
    serde_json,
};

use crate::{id::Id, parameter::RpcParameter};

#[derive(Debug, Serialize)]
#[serde(crate = "sequencer_core::serde")]
pub struct RpcRequest<T>
where
    T: RpcParameter,
{
    jsonrpc: &'static str,
    method: &'static str,
    params: T,
    id: Id,
}

impl<T> From<(T, Id)> for RpcRequest<T>
where
    T: RpcParameter,
{
    fn from(value: (T, Id)) -> Self {
        Self {
            jsonrpc: "2.0",
            method: T::method_name(),
            params: value.0,
            id: value.1,
        }
    }
}

impl<T> RpcRequest<T>
where
    T: RpcParameter,
{
    pub fn to_json_string(&self) -> Result<String, Error> {
        serde_json::to_string(self).wrap(caller!(RpcRequest::to_json_string()))
    }
}
