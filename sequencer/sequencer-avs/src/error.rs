#[derive(Debug)]
pub enum Error {
    OpenConfig(std::io::Error),
    ParseConfig(toml::de::Error),
    Ssal(ssal::ethereum::Error),
    Seeder(ssal::ethereum::Error),
    Database(database::Error),
    JsonRPC(json_rpc::Error),
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
