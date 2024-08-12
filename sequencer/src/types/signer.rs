use std::collections::HashMap;

use radius_sequencer_sdk::liveness::types::Address as AlloyAddress;

use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SigningKey(String);

impl SigningKey {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }

    // TODO: change
    pub fn get_address(&self) -> Address {
        if self.0 == "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" {
            return Address::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        } else if self.0 == "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff81" {
            return Address::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92267");
        } else if self.0 == "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff82" {
            return Address::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92268");
        } else if self.0 == "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff83" {
            return Address::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92269");
        } else if self.0 == "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff84" {
            return Address::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92270");
        }

        Address::new("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92271")
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
