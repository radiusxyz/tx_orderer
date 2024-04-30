use std::{any::type_name, fmt::Debug, sync::Arc};

use rocksdb::TransactionDB;
use serde::{de::DeserializeOwned, ser::Serialize};

use crate::{
    context,
    database::lock::Lock,
    error::{Error, WrapError},
};

pub struct Database {
    client: Arc<TransactionDB>,
}

unsafe impl Send for Database {}

unsafe impl Sync for Database {}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl Database {
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
        let key_vec = bincode::serialize(key).wrap(context!(key))?;

        let value_slice = self
            .client
            .get_pinned(key_vec)
            .wrap(context!(key))?
            .wrap(context!(key))?;

        let value: V =
            bincode::deserialize(value_slice.as_ref()).wrap(context!(key, type_name::<V>()))?;
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
        let key_vec = bincode::serialize(key).wrap(context!(key))?;
        let transaction = self.client.transaction();

        let value_vec = transaction
            .get_for_update(&key_vec, true)
            .wrap(context!(key))?
            .wrap(context!(key))?;

        let value: V =
            bincode::deserialize(value_vec.as_ref()).wrap(context!(key, type_name::<V>()))?;

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
        let key_vec = bincode::serialize(key).wrap(context!(key))?;

        let value_vec = bincode::serialize(value).wrap(context!(value))?;

        let transaction = self.client.transaction();
        transaction
            .put(key_vec, value_vec)
            .wrap(context!(key, value))?;
        transaction.commit().wrap(context!(key, value))?;
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
        let key_vec = bincode::serialize(key).wrap(context!(key))?;

        let transaction = self.client.transaction();
        transaction.delete(key_vec).wrap(context!(key))?;
        transaction.commit().wrap(context!(key))?;
        Ok(())
    }
}
