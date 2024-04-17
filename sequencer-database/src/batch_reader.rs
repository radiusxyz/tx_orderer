use std::{any::type_name, collections::HashMap, fmt::Debug};

use sequencer_core::{
    bincode, context,
    error::{Error, WrapError},
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
        let key_vec = bincode::serialize(key).wrap(context!(key))?;

        let value_vec_opt = self.key_value_map.get(&key_vec).wrap(context!(key))?;
        let value_vec = value_vec_opt.as_ref().wrap(context!(key))?;
        let value: V = bincode::deserialize(&value_vec).wrap(context!(key, type_name::<V>()))?;
        Ok(value)
    }
}
