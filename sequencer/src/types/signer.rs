use std::collections::HashMap;

use radius_sequencer_sdk::liveness::types::Address as AlloyAddress;

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SigningKey(String);

impl SigningKey {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }
}

impl From<String> for SigningKey {
    fn from(signing_key: String) -> Self {
        Self(signing_key)
    }
}

impl From<&str> for SigningKey {
    fn from(signing_key: &str) -> Self {
        Self(signing_key.to_owned())
    }
}

impl From<SigningKey> for String {
    fn from(signing_key: SigningKey) -> String {
        signing_key.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicKey(String);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Address(String);

impl Default for Address {
    fn default() -> Self {
        Self("".to_string())
    }
}

impl Address {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<String> for Address {
    fn from(address: String) -> Self {
        Self(address)
    }
}

impl From<&str> for Address {
    fn from(address: &str) -> Self {
        Self(address.to_owned())
    }
}

// TODO:
impl PartialEq<AlloyAddress> for Address {
    fn eq(&self, other: &AlloyAddress) -> bool {
        true
    }
}

pub type AddressList = Vec<Address>;

impl PartialEq<Address> for AlloyAddress {
    fn eq(&self, other: &Address) -> bool {
        other == self // AddressString에 대한 PartialEq 구현을 사용합니다.
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Signature {
    r: String,
    s: String,
    v: String,
}

impl Default for Signature {
    fn default() -> Self {
        Self {
            r: "".to_string(),
            s: "".to_string(),
            v: "".to_string(),
        }
    }
}
