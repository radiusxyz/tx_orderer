use sequencer_core::{
    bincode, context,
    error::{Error, WrapError},
    rocksdb::{Transaction, TransactionDB},
    serde::ser::Serialize,
};

/// A locking mechanism for values stored in the database.
///
/// This struct provides automatic persistence when mutated. When the `Lock` goes out of scope,
/// the held value is automatically serialized and stored back to the database within the associated transaction.
pub struct Lock<'db, V>
where
    V: std::fmt::Debug + Serialize,
{
    transaction: Option<Transaction<'db, TransactionDB>>,
    key_vec: Vec<u8>,
    value: V,
}

impl<'db, V> std::ops::Deref for Lock<'db, V>
where
    V: std::fmt::Debug + Serialize,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'db, V> std::ops::DerefMut for Lock<'db, V>
where
    V: std::fmt::Debug + Serialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'db, V> Lock<'db, V>
where
    V: std::fmt::Debug + Serialize,
{
    pub fn new(
        transaction: Option<Transaction<'db, TransactionDB>>,
        key_vec: Vec<u8>,
        value: V,
    ) -> Self {
        Self {
            transaction,
            key_vec,
            value,
        }
    }

    /// Commit any changes made to inner V to the database.
    ///
    /// # Arguments
    ///
    /// * `self` - consumes `self` so inner data cannot be accessed or modified after the function call.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there are any issues serializing the value or with the database operation itself.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use database::Database;
    /// use serde::{Deserialize, Serialize};
    ///
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
    pub fn commit(mut self) -> Result<(), Error> {
        let value = &self.value;

        if let Some(transaction) = self.transaction.take() {
            let value_vec = bincode::serialize(value).wrap(context!(value))?;

            transaction
                .put(&self.key_vec, value_vec)
                .wrap(context!(value))?;
            transaction.commit().wrap(context!(value))?;
        }
        Ok(())
    }
}
