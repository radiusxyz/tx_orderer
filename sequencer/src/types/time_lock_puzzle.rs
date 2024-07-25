use crate::types::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeLockPuzzle {
    o: String,
    t: u32,
    n: String,
}

impl TimeLockPuzzle {
    pub fn new(t: u32, o: impl AsRef<str>, n: impl AsRef<str>) -> Self {
        Self {
            o: o.as_ref().to_owned(),
            t,
            n: n.as_ref().to_owned(),
        }
    }
}
