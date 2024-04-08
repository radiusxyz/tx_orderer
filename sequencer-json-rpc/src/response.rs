use std::{error, fmt};

use sequencer_core::serde::Deserialize;

use crate::id::Id;

#[derive(Debug, Deserialize)]
#[serde(crate = "sequencer_core::serde")]
#[serde(untagged)]
pub enum RpcResponse<T> {
    Result {
        jsonrpc: String,
        result: T,
        id: Id,
    },
    Error {
        jsonrpc: String,
        error: RpcErrorMessage,
        id: Id,
    },
}

#[derive(Deserialize)]
#[serde(crate = "sequencer_core::serde")]
pub struct RpcErrorMessage {
    code: i16,
    message: String,
    data: Option<String>,
}

impl fmt::Debug for RpcErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for RpcErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code: {}, message: {}", self.code, self.message)
    }
}

impl error::Error for RpcErrorMessage {}

impl RpcErrorMessage {
    pub fn code(&self) -> i16 {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn data(&self) -> &Option<String> {
        &self.data
    }
}
