use std::{any, collections::HashMap, fmt::Debug};

use sequencer_core::{
    bincode, caller,
    error::{DatabaseError, Error, ErrorKind, WrapError},
    serde::{de::DeserializeOwned, ser::Serialize},
};

#[derive(Default)]
pub struct BatchReader {
    key_value_map: HashMap<Vec<u8>, Option<Vec<u8>>>,
}

impl From<HashMap<Vec<u8>, Option<Vec<u8>>>> for BatchReader {
    fn from(value: HashMap<Vec<u8>, Option<Vec<u8>>>) -> Self {
        Self {
            key_value_map: value,
        }
    }
}

impl BatchReader {
    pub fn get<K, V>(&self, key: &K) -> Result<V, Error>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned,
    {
        let key_vec = bincode::serialize(key)
            .map_err(ErrorKind::from)
            .wrap(DatabaseError::SerializeKey(key))?;

        let value_vec_opt = self
            .key_value_map
            .get(&key_vec)
            .ok_or(ErrorKind::NoneType)
            .wrap(DatabaseError::KeyDoesNotExist(key))?;
        let value_vec = value_vec_opt.as_ref().ok_or(ErrorKind::NoneType).wrap("")?;
        // .wrap_context(caller!(BatchReader::get()), format_args!("key: {:?}", key))?;
        let value: V = bincode::deserialize(&value_vec)
            .map_err(ErrorKind::from)
            .wrap(DatabaseError::PutTransaction(key))?;
        Ok(value)
    }
}
