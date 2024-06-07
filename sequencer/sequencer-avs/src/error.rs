#[derive(Debug)]
pub enum Error {
    OpenConfig(std::io::Error),
    ParseConfig(toml::de::Error),
    Database(database::Error),
    JsonRPC(json_rpc::Error),
    Ssal(ssal::ethereum::Error),
    Uninitialized,
    EmptyLeaderAddress,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<database::Error> for Error {
    fn from(value: database::Error) -> Self {
        Self::Database(value)
    }
}

impl From<json_rpc::Error> for Error {
    fn from(value: json_rpc::Error) -> Self {
        Self::JsonRPC(value)
    }
}

impl From<ssal::ethereum::Error> for Error {
    fn from(value: ssal::ethereum::Error) -> Self {
        Self::Ssal(value)
    }
}
