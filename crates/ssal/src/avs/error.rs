#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    BuildSeederClient,
    ParseRpcUrl,
    ParseSigningKey,
    ParseSsalContractAddress,
    ParseDelegationManagerContractAddress,
    ParseStakeRegistryContractAddress,
    ParseAvsDirectoryContractAddress,
    ParseAvsContractAddress,
    ParseSequencerAddress,
    ParseRollupAddress,
    ParseBlockCommitment,
    ParseClusterId,
    ParseRollupId,
    ParseMessageToBytes,
    GetBlockNumber,
    IsOperator,
    RegisterAsOperator,
    IsAvs,
    CalculateDigestHash,
    OperatorSignature,
    RegisterOnAvs,
    DeregisterOperator,
    IsRegistered,
    InitializeCluster,
    RegisterSequencer,
    DeregisterSequencer,
    RegisterBlockCommitment,
    RespondToTask,
    GetSequencerAddress,
    EmptySequencerList,
    GetSequencerRpcUrl,
    Keystore,
    ConnectEventListener,
    BlockStream,
    InitializeClusterEventStream,
    BlockCommitmentEventStream,
    EventListener,
    SignTask,
}

pub enum ErrorSource {
    Boxed(Box<dyn std::error::Error>),
    Custom(String),
    IO(std::io::Error),
    JsonRPC(radius_sequencer_sdk::json_rpc::Error),
    LocalSigner(alloy::signers::local::LocalSignerError),
    Hex(alloy::hex::FromHexError),
    Contract(alloy::contract::Error),
    Transport(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    Signer(alloy::signers::Error),
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boxed(error) => write!(f, "{}", error),
            Self::Custom(error) => write!(f, "{}", error),
            Self::IO(error) => write!(f, "{}", error),
            Self::JsonRPC(error) => write!(f, "{}", error),
            Self::LocalSigner(error) => write!(f, "{}", error),
            Self::Hex(error) => write!(f, "{}", error),
            Self::Contract(error) => write!(f, "{}", error),
            Self::Transport(error) => write!(f, "{}", error),
            Self::Signer(error) => write!(f, "{}", error),
        }
    }
}

impl From<std::io::Error> for ErrorSource {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<radius_sequencer_sdk::json_rpc::Error> for ErrorSource {
    fn from(value: radius_sequencer_sdk::json_rpc::Error) -> Self {
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

impl From<alloy::signers::Error> for ErrorSource {
    fn from(value: alloy::signers::Error) -> Self {
        Self::Signer(value)
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

impl From<(ErrorKind, Box<dyn std::error::Error>)> for Error {
    fn from(value: (ErrorKind, Box<dyn std::error::Error>)) -> Self {
        Self {
            kind: value.0,
            source: ErrorSource::Boxed(value.1),
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
