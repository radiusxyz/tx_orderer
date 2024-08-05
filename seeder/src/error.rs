pub use radius_sequencer_sdk::kvstore::KvStoreError as DbError;

pub enum Error {
    Boxed(Box<dyn std::error::Error>),
    OpenConfig(std::io::Error),
    ParseConfig(toml::de::Error),
    Database(DbError),
    JsonRPC(radius_sequencer_sdk::json_rpc::Error),
    SignatureMismatch,
    HealthCheck,

    RemoveConfigDirectory,
    CreateConfigDirectory,
    CreateConfigFile,
    CreatePrivateKeyFile,
    LoadConfigOption,
    ParseTomlString,

    ConnectEventListener,
    ParseContractAddress,
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
            Self::RemoveConfigDirectory => {
                write!(f, "Failed to remove the previous configuration directory")
            }
            Self::CreateConfigDirectory => {
                write!(f, "Failed to create a new configuration directory")
            }
            Self::CreateConfigFile => {
                write!(f, "Failed to create a new config file")
            }
            Self::CreatePrivateKeyFile => {
                write!(f, "Failed to create a private key file")
            }
            Self::LoadConfigOption => {
                write!(f, "Failed to load a config file")
            }
            Self::ParseTomlString => {
                write!(f, "Failed to parse String to TOML String")
            }
            Self::ConnectEventListener => {
                write!(f, "Failed to connect to the event listener")
            }
            Self::ParseContractAddress => {
                write!(f, "Failed to parse contract address")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<DbError> for Error {
    fn from(value: DbError) -> Self {
        Self::Database(value)
    }
}

impl From<radius_sequencer_sdk::json_rpc::Error> for Error {
    fn from(value: radius_sequencer_sdk::json_rpc::Error) -> Self {
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
