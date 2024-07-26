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

impl Default for Signature {
    fn default() -> Self {
        Self {
            r: "".to_string(),
            s: "".to_string(),
            v: "".to_string(),
        }
    }
}
