use std::fmt::Debug;

use rocksdb::{Transaction, TransactionDB};
use serde::ser::Serialize;

use crate::{Error, ErrorKind};

/// A locking mechanism for values stored in the database.
///
/// This struct provides automatic persistence when mutated. When the `Lock` goes out of scope,
/// the held value is automatically serialized and stored back to the database within the associated transaction.
pub struct Lock<'db, V>
where
    V: Debug + Serialize,
{
    transaction: Option<Transaction<'db, TransactionDB>>,
    key_vec: Vec<u8>,
    value: V,
}

impl<'db, V> std::ops::Deref for Lock<'db, V>
where
    V: Debug + Serialize,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'db, V> std::ops::DerefMut for Lock<'db, V>
where
    V: Debug + Serialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<'db, V> Lock<'db, V>
where
    V: Debug + Serialize,
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

    pub fn commit(mut self) -> Result<(), Error> {
        if let Some(transaction) = self.transaction.take() {
            let value_vec = bincode::serialize(&self.value)
                .map_err(|error| (ErrorKind::SerializeValue, error))?;

            transaction
                .put(&self.key_vec, value_vec)
                .map_err(|error| (ErrorKind::Put, error))?;
            transaction
                .commit()
                .map_err(|error| (ErrorKind::Commit, error))?;
        }
        Ok(())
    }
}
