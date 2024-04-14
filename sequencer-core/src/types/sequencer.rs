// use std::collections::{hash_set::Iter, HashSet};

// use crate::{
//     caller,
//     error::Error,
//     rand::{self, seq::SliceRandom},
//     serde::{Deserialize, Serialize},
//     types::*,
// };

// #[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
// pub struct SequencerId {
//     id: String,
//     address: String,
// }

// #[derive(Clone, Debug, Default, Deserialize, Serialize)]
// pub struct SequencerSet {
//     block_height: BlockHeight,
//     set: HashSet<SequencerId>,
//     leader: Option<SequencerId>,
// }

// impl SequencerSet {
//     pub fn new(block_height: BlockHeight) -> Self {
//         Self {
//             block_height,
//             set: HashSet::default(),
//             leader: None,
//         }
//     }

//     pub fn register(&mut self, sequencer_id: SequencerId) -> Result<(), Error> {
//         match self.set.insert(sequencer_id) {
//             true => Ok(()),
//             false => Err(Error::str_error(
//                 caller!(SequencerSet::register()),
//                 "Sequencer is already registered",
//             )),
//         }
//     }

//     pub fn elect_leader(&mut self) -> Result<SequencerId, Error> {
//         let sequencer_vec: Vec<SequencerId> = self.set.iter().cloned().collect();
//         match sequencer_vec.choose(&mut rand::thread_rng()) {
//             Some(leader) => {
//                 self.leader = Some(leader.clone());
//                 Ok(leader.clone())
//             }
//             None => Err(Error::str_error(
//                 caller!(SequencerSet::elect_leader()),
//                 "Failed to elect the leader.",
//             )),
//         }
//     }

//     pub fn leader(&self) -> Option<SequencerId> {
//         match &self.leader {
//             Some(leader) => Some(leader.clone()),
//             None => None,
//         }
//     }

//     pub fn iter<'a>(&'a self) -> Iter<'a, SequencerId> {
//         self.set.iter()
//     }
// }
