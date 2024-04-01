use core::rocksdb::TransactionDB;
use std::{path::Path, sync::Arc};

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
    // pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {}
}
