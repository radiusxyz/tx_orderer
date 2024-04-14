pub enum DatabaseError<'a, T>
where
    T: std::fmt::Debug,
{
    SerializeKey(&'a T),
    SerializeValue(&'a T),
    GetBytesValue(&'a T),
    KeyDoesNotExist(&'a T),
    DeserializeValue(&'a T, &'static str),
    PutTransaction(&'a T),
    CommitTransaction(&'a T),
}

impl<'a, T> std::fmt::Debug for DatabaseError<'a, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializeKey(key) => write!(f, "DatabaseError::SerializeKey({:?})", key),
            Self::SerializeValue(value) => {
                write!(f, "DatabaseError::SerializeValue ({:?})", value)
            }
            Self::GetBytesValue(key) => write!(f, "DatabaseError::GetBytesValue (key: {:?})", key)?,
            Self::KeyDoesNotExist(key) => write!(f, "Key: {:?} does not exist", key)?,
            Self::DeserializeValue(key, value_type) => write!(
                f,
                "Failed to deserialize the value into {:?} for the key ({:?})",
                value_type, key
            )?,
            Self::PutTransaction(key) => write!(f, ""),
        }
    }
}
