#[derive(Debug)]
pub enum Error {
    Open(rocksdb::Error),
    Get(rocksdb::Error),
    GetMut(rocksdb::Error),
    Put(rocksdb::Error),
    Delete(rocksdb::Error),
    Commit(rocksdb::Error),
    SerializeKey(bincode::Error),
    SerializeValue(bincode::Error),
    DeserializeValue(bincode::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
