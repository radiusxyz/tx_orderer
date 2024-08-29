use crate::types::prelude::*;

pub type IpAddress = String;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SequencerStatus {
    Uninitialized,
    Running,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct SequencerList(Vec<(Address, Option<String>)>);

// impl From<(Vec<Address>, Vec<Option<String>>)> for SequencerList {
//     fn from(value: (Vec<Address>, Vec<Option<String>>)) -> Self {
//         Self(std::iter::zip(value.0, value.1).collect())
//     }
// }

// impl From<Vec<(Address, Option<String>)>> for SequencerList {
//     fn from(value: Vec<(Address, Option<String>)>) -> Self {
//         Self(value)
//     }
// }
