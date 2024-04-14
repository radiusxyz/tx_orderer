use std::collections::HashMap;

use sequencer_core::{
    caller,
    _error::{Error, WrapError},
};

use crate::batch_reader::BatchReader;

type KeyList = Result<Vec<Vec<u8>>, Error>;
type KeyValueList = Result<Vec<(Vec<u8>, Vec<u8>)>, Error>;

impl super::Database {
    /// Get `BatchReader` from which a user can get a value by a given key.
    ///
    /// Rather than implementing KeyList type, it is advised to use `key_list!()` macro to pass parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    /// pub struct A(String);
    ///
    /// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    /// pub struct B(String);
    ///
    /// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    /// pub struct C(String);
    ///
    /// let database = Database::new("../../test_database").unwrap();
    /// database
    ///     .batch_put(key_value_list![
    ///         (&"Key A", &A(String::from("This is A"))),
    ///         (&2, &B(String::from("This is B"))),
    ///         (&3, &C(String::from("This is C")))
    ///     ])
    ///     .unwrap();
    ///
    /// let batch_reader = database.batch_get(key_list!(&"Key A", &2, &3)).unwrap();
    ///
    /// let value_a: A = batch_reader.get(&"Key A").unwrap();
    ///
    /// assert!(value_a == A(String::from("This is A")));
    ///
    /// let value_b: B = batch_reader.get(&2).unwrap();
    ///
    /// assert!(value_b == B(String::from("This is B")));
    ///
    /// let value_c: C = batch_reader.get(&3).unwrap();
    ///
    /// assert!(value_c == C(String::from("This is C")));
    /// ```
    pub fn batch_get(&self, key_list: KeyList) -> Result<BatchReader, Error> {
        match key_list {
            Ok(key_list) => {
                let mut key_value_map = HashMap::<Vec<u8>, Option<Vec<u8>>>::default();

                let value_list = self.client.multi_get(&key_list);

                for (key_vec, value_wrapped) in key_list.into_iter().zip(value_list) {
                    let value_option = value_wrapped.wrap(caller!(Database::batch_get()))?;
                    key_value_map.insert(key_vec, value_option);
                }
                Ok(key_value_map.into())
            }
            Err(error) => Err(error),
        }
    }

    /// Batch-put a list of key-value pairs to the database.
    ///
    /// Rather than implementing KeyValueList type, it is advised to use `key_value_list!()` macro to pass parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use primitives::serde::{Deserialize, Serialize};
    ///
    /// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    /// pub struct A(String);
    ///
    /// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    /// pub struct B(String);
    ///
    /// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    /// pub struct C(String);
    ///
    /// let database = Database::new("../../test_database").unwrap();
    /// database
    ///     .batch_put(key_value_list![
    ///         (&"Key A", &A(String::from("This is A"))),
    ///         (&2, &B(String::from("This is B"))),
    ///         (&3, &C(String::from("This is C")))
    ///     ])
    ///     .unwrap();
    ///
    /// let batch_reader = database.batch_get(key_list!(&"Key A", &2, &3)).unwrap();
    ///
    /// let value_a: A = batch_reader.get(&"Key A").unwrap();
    ///
    /// assert!(value_a == A(String::from("This is A")));
    ///
    /// let value_b: B = batch_reader.get(&2).unwrap();
    ///
    /// assert!(value_b == B(String::from("This is B")));
    ///
    /// let value_c: C = batch_reader.get(&3).unwrap();
    ///
    /// assert!(value_c == C(String::from("This is C")));
    /// ```
    pub fn batch_put(&self, key_value_list: KeyValueList) -> Result<(), Error> {
        match key_value_list {
            Ok(key_value_list) => {
                let transaction = self.client.transaction();

                for (key, value) in key_value_list {
                    transaction
                        .put(&key, &value)
                        .wrap(caller!(Database::batch_put()))?;
                }

                transaction.commit().wrap(caller!(Database::batch_put()))?;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Batch-delete a list of keys from the database.
    ///
    /// # Examples
    /// ```
    /// use primitives::serde::{Deserialize, Serialize};
    ///
    /// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    /// pub struct A(String);
    ///
    /// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    /// pub struct B(String);
    ///
    /// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    /// pub struct C(String);
    ///
    /// let database = crate::Database::new("../../test_database").unwrap();
    /// database
    ///     .batch_put(crate::key_value_list![
    ///     (&"Key A", &A(String::from("This is A"))),
    ///     (&2, &B(String::from("This is B"))),
    ///     (&3, &C(String::from("This is C")))
    /// ])
    /// .unwrap();
    ///
    /// database
    ///     .batch_delete(crate::key_list!(&"Key A", &2))
    ///     .unwrap();
    ///
    /// let batch_reader = database
    ///     .batch_get(crate::key_list!(&"Key A", &2, &3))
    ///     .unwrap();
    ///
    /// assert!(batch_reader
    ///     .get::<&str, A>(&"Key A")
    ///     .is_err_and(|error| error == crate::DatabaseError::ValueReturnedNone));
    ///
    /// assert!(batch_reader
    ///     .get::<i32, B>(&2)
    ///     .is_err_and(|error| error == crate::DatabaseError::ValueReturnedNone));
    ///
    /// let value_c: C = batch_reader.get(&3).unwrap();
    ///
    /// assert!(value_c == C(String::from("This is C")));
    /// ```
    pub fn batch_delete(&self, key_list: KeyList) -> Result<(), Error> {
        match key_list {
            Ok(key_list) => {
                let transaction = self.client.transaction();

                for key_vec in key_list {
                    transaction
                        .delete(key_vec)
                        .wrap(caller!(Database::batch_delete()))?;
                }

                transaction
                    .commit()
                    .wrap(caller!(Database::batch_delete()))?;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}
