pub enum Error {
    Boxed(Box<dyn std::error::Error>),
    OpenConfig(std::io::Error),
    ParseConfig(toml::de::Error),
    Database(database::Error),
    JsonRPC(json_rpc::Error),
    SignatureMismatch,
    HealthCheck,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boxed(error) => write!(f, "{}", error),
            Self::OpenConfig(error) => write!(f, "{}", error),
            Self::ParseConfig(error) => write!(f, "{}", error),
            Self::Database(error) => write!(f, "{}", error),
            Self::JsonRPC(error) => write!(f, "{}", error),
            Self::SignatureMismatch => write!(f, "Sender is not the signer."),
            Self::HealthCheck => {
                write!(
                    f,
                    "Health-check failed. Make sure the sequencer is running and port-forwarded."
                )
            }
        }
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

impl Error {
    pub fn boxed<E>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self::Boxed(Box::new(error))
    }
}
