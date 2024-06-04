use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RpcAddress(String);

impl AsRef<[u8]> for RpcAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<str> for RpcAddress {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for RpcAddress {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<&String> for RpcAddress {
    fn from(value: &String) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for RpcAddress {
    fn from(value: String) -> Self {
        Self(value)
    }
}
