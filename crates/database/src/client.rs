use std::{fmt::Debug, path::Path, sync::Arc};

use primitives::{
    bincode,
    error::{Error, ErrorKind},
    rocksdb::{Options, TransactionDB, TransactionDBOptions},
    serde::{de::DeserializeOwned, ser::Serialize},
};

use crate::lock::Lock;

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
                .map_err(Error::new)?;
        Ok(Self {
            client: Arc::new(transaction_database),
        })
    }
    /// Retrieves a value associated with the given key from the database.
    ///
    /// # Arguments
    ///
    /// * `key` - The key whose associated value is to be retrieved.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there are any issues serializing the key or deserializing the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let db = Database::new("/path/to/db").unwrap();
    /// let value: MyType = match db.get(&my_key) {
    ///     Ok(my_value) => my_value,
    ///     Err(error) => {
    ///         println!("{}", error);
    ///         MyType::default()
    ///     },
    /// }
    /// ```
    pub fn get<K, V>(&self, key: &K) -> Result<V, Error>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(Error::new)?;

        let value_slice = self
            .client
            .get_pinned(key_vec)
            .map_err(Error::new)?
            .ok_or(ErrorKind::NoneType)?;

        let value: V = bincode::deserialize(value_slice.as_ref()).map_err(Error::new)?;
        Ok(value)
    }

    /// Retrieves a mutable reference (locked) to the value associated with the given key from the database.
    ///
    /// The returned `Lock<V>` wraps a mutable reference to the value and ensures exclusive access.
    /// This means that the value is locked until the `Lock` is dropped, preventing concurrent modifications.
    ///
    /// # Arguments
    ///
    /// * `key` - The key whose associated value is to be retrieved.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there are any issues serializing the key or deserializing the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[derive(Debug, Deserialize, Serialize)]
    /// pub struct MyType {
    ///     name: String,
    /// }
    ///
    /// let db = Database::new("/path/to/db").unwrap();
    ///
    /// // Get owned mutable reference to `MyType`.
    /// let mut locked_value: Lock<MyType> = db.get_mut(&key).unwrap();
    /// locked_value.name = String::from("foo");
    ///
    /// // After calling commit(), `MyType` is no longer accessible or modifiable.
    /// locked_value.commit().unwrap();
    /// ```
    pub fn get_mut<K, V>(&self, key: &K) -> Result<Lock<V>, Error>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(Error::new)?;
        let transaction = self.client.transaction();

        let value_vec = transaction
            .get_for_update(&key_vec, true)
            .map_err(Error::new)?
            .ok_or(ErrorKind::NoneType)?;

        let value: V = bincode::deserialize(value_vec.as_ref()).map_err(Error::new)?;

        let locked_value = Lock::new(Some(transaction), key_vec, value);
        Ok(locked_value)
    }

    /// Inserts or updates the value associated with the given key in the database.
    ///
    /// If the database previously contained a value for the given key, the old value is replaced.
    ///
    /// # Arguments
    ///
    /// * `key` - The key under which the value will be stored.
    /// * `value` - The value to be stored associated with the provided key.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there are any issues serializing the key or value, or with the database operation itself.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let db = Database::new("/path/to/db").unwrap();
    /// db.put(&my_key, &my_value).unwrap();
    /// ```
    pub fn put<K, V>(&self, key: &K, value: &V) -> Result<(), Error>
    where
        K: Debug + Serialize,
        V: Debug + DeserializeOwned + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(Error::new)?;

        let value_vec = bincode::serialize(value).map_err(Error::new)?;

        let transaction = self.client.transaction();
        transaction.put(key_vec, value_vec).map_err(Error::new)?;
        transaction.commit().map_err(Error::new)?;
        Ok(())
    }

    /// Removes the value associated with the given key in the database, if it exists.
    ///
    /// If the key does not exist in the database, this function will do nothing.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the value to be removed from the database.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there are any issues serializing the key or with the database operation itself.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let db = Database::new("/path/to/db").unwrap();
    /// db.delete(&my_key).unwrap();
    /// ```
    pub fn delete<K>(&self, key: &K) -> Result<(), Error>
    where
        K: Debug + Serialize,
    {
        let key_vec = bincode::serialize(key).map_err(Error::new)?;

        let transaction = self.client.transaction();
        transaction.delete(key_vec).map_err(Error::new)?;
        transaction.commit().map_err(Error::new)?;
        Ok(())
    }
}
