use serde::{Deserialize, Serialize};
use ssal::ethereum::PublicKey;

use crate::types::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SequencerStatus {
    Uninitialized,
    Initialized,
    BlockBuildingInProgress,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerAddress(String);

impl AsRef<[u8]> for SequencerAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<str> for SequencerAddress {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for SequencerAddress {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerListKey(&'static str, SsalBlockNumber);

impl SequencerListKey {
    const IDENTIFIER: &'static str = stringify!(SequencerListKey);

    pub fn new(ssal_block_number: SsalBlockNumber) -> Self {
        Self(Self::IDENTIFIER, ssal_block_number)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerList(Vec<(PublicKey, Option<SequencerAddress>)>);

impl SequencerList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> core::slice::Iter<(PublicKey, Option<SequencerAddress>)> {
        self.0.iter()
    }
}
