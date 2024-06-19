#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    BuildSeederClient,
    InitializeCluster,
    RegisterSequencer,
    DeregisterSequencer,
    RegisterBlockCommitment,
    GetSequencerRpcUrl,

    ParseRpcUrl,
    ParseContractAddress,
    ParseSequencerAddress,
    ParseRollupAddress,
    ParseClusterID,
    Keystore,
    ConnectEventListener,
    BlockStream,
    InitializeClusterEventStream,
    BlockCommitmentEventStream,
    EventListener,
}

pub enum ErrorSource {
    Boxed(Box<dyn std::error::Error>),
    Custom(String),
    JsonRPC(json_rpc::Error),
    LocalSigner(alloy::signers::local::LocalSignerError),
    Hex(alloy::hex::FromHexError),
    Contract(alloy::contract::Error),
    Transport(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boxed(error) => write!(f, "{}", error),
            Self::Custom(error) => write!(f, "{}", error),
            Self::JsonRPC(error) => write!(f, "{}", error),
            Self::LocalSigner(error) => write!(f, "{}", error),
            Self::Hex(error) => write!(f, "{}", error),
            Self::Contract(error) => write!(f, "{}", error),
            Self::Transport(error) => write!(f, "{}", error),
        }
    }
}

impl From<json_rpc::Error> for ErrorSource {
    fn from(value: json_rpc::Error) -> Self {
        Self::JsonRPC(value)
    }
}

impl From<alloy::signers::local::LocalSignerError> for ErrorSource {
    fn from(value: alloy::signers::local::LocalSignerError) -> Self {
        Self::LocalSigner(value)
    }
}

impl From<alloy::hex::FromHexError> for ErrorSource {
    fn from(value: alloy::hex::FromHexError) -> Self {
        Self::Hex(value)
    }
}

impl From<alloy::contract::Error> for ErrorSource {
    fn from(value: alloy::contract::Error) -> Self {
        Self::Contract(value)
    }
}

impl From<alloy::transports::RpcError<alloy::transports::TransportErrorKind>> for ErrorSource {
    fn from(value: alloy::transports::RpcError<alloy::transports::TransportErrorKind>) -> Self {
        Self::Transport(value)
    }
}

pub struct Error {
    kind: ErrorKind,
    source: ErrorSource,
}

unsafe impl Send for Error {}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.source)
    }
}

impl std::error::Error for Error {}

impl<E> From<(ErrorKind, E)> for Error
where
    E: Into<ErrorSource>,
{
    fn from(value: (ErrorKind, E)) -> Self {
        Self {
            kind: value.0,
            source: value.1.into(),
        }
    }
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn boxed<E>(kind: ErrorKind, error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            kind,
            source: ErrorSource::Boxed(Box::new(error)),
        }
    }

    pub fn custom<E>(kind: ErrorKind, error: E) -> Self
    where
        E: std::fmt::Display,
    {
        Self {
            kind,
            source: ErrorSource::Custom(error.to_string()),
        }
    }
}
