#[derive(Debug)]
pub enum Error {
    OpenConfig(std::io::Error),
    ParseConfig(toml::de::Error),
    Ssal(ssal::ethereum::Error),
    Seeder(ssal::ethereum::Error),
    Database(database::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
