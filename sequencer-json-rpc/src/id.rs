use sequencer_core::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "sequencer_core::serde")]
#[serde(untagged)]
pub enum Id {
    String(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<u8> for Id {
    fn from(value: u8) -> Self {
        Self::U8(value)
    }
}

impl From<u16> for Id {
    fn from(value: u16) -> Self {
        Self::U16(value)
    }
}

impl From<u32> for Id {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<u64> for Id {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<i8> for Id {
    fn from(value: i8) -> Self {
        Self::I8(value)
    }
}

impl From<i16> for Id {
    fn from(value: i16) -> Self {
        Self::I16(value)
    }
}

impl From<i32> for Id {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}
