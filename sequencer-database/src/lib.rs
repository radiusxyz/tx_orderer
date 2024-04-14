pub mod batch_reader;
pub mod impl_basic_ops;
pub mod impl_batch_ops;
pub mod lock;
pub mod macros;

use std::{path::Path, sync::Arc};

use sequencer_core::{
    caller,
    _error::{Error, WrapError},
    rocksdb::{Options, TransactionDB, TransactionDBOptions},
};

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
    /// Constructs a new `Database` instance, opening or creating it at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A path reference where the database should be opened or created.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there are any issues opening or creating the database.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let db = Database::new("/path/to/db").unwrap();
    /// ```
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut db_options = Options::default();
        db_options.create_if_missing(true);
        let tx_db_options = TransactionDBOptions::default();

        let transaction_db = TransactionDB::open(&db_options, &tx_db_options, &path).wrap_context(
            caller!(Database::new()),
            format_args!("path: {:?}", path.as_ref()),
        )?;

        Ok(Self {
            client: Arc::new(transaction_db),
        })
    }
}
