use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SigningKey(String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicKey(String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Address(String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Signature {
    r: String,
    s: String,
    v: String,
}
