use std::{fmt::Debug, path::Path, sync::Arc};

use rocksdb::{Options, TransactionDB, TransactionDBOptions};
use serde::{de::DeserializeOwned, ser::Serialize};

use crate::{Error, Lock};

pub struct Database {
    client: Arc<TransactionDB>,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut database_options = Options::default();
        database_options.create_if_missing(true);

        let transaction_database_options = TransactionDBOptions::default();
        let transaction_database =
            TransactionDB::open(&database_options, &transaction_database_options, path)
                .map_err(Error::Open)?;
        Ok(Self {
            client: Arc::new(transaction_database),
        })
    }

    pub fn get<K, V>(&self, key: &K) -> Result<Option<V>, Error>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(Error::SerializeKey)?;

        match self.client.get_pinned(key_vec).map_err(Error::Get)? {
            Some(value_slice) => {
                let value: V =
                    bincode::deserialize(value_slice.as_ref()).map_err(Error::DeserializeValue)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn get_mut<K, V>(&self, key: &K) -> Result<Option<Lock<V>>, Error>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(Error::SerializeKey)?;

        let transaction = self.client.transaction();
        match transaction
            .get_for_update(&key_vec, true)
            .map_err(Error::GetMut)?
        {
            Some(value_vec) => {
                let value: V = bincode::deserialize(&value_vec).map_err(Error::DeserializeValue)?;
                let locked_value = Lock::new(Some(transaction), key_vec, value);
                Ok(Some(locked_value))
            }
            None => Ok(None),
        }
    }

    pub fn put<K, V>(&self, key: &K, value: &V) -> Result<(), Error>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(Error::SerializeKey)?;
        let value_vec = bincode::serialize(value).map_err(Error::SerializeValue)?;

        let transaction = self.client.transaction();
        transaction.put(key_vec, value_vec).map_err(Error::Put)?;
        transaction.commit().map_err(Error::Commit)?;
        Ok(())
    }

    pub fn delete<K>(&self, key: &K) -> Result<(), Error>
    where
        K: Debug + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(Error::SerializeKey)?;

        let transaction = self.client.transaction();
        transaction.delete(key_vec).map_err(Error::Delete)?;
        transaction.commit().map_err(Error::Commit)?;
        Ok(())
    }
}
