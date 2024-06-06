#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    BuildSeederClient,
    ParseSeederAddress,
    Register,
    Deregister,
    GetAddressList,
    DeserializeResponse,
    BuildSsalClient,
    ParseContractAddress,
    GetBlockNumber,
    GetSequencerList,
    ParseStr,
    ParseToPublicKey,
}

pub enum ErrorSource {
    Boxed(Box<dyn std::error::Error>),
    JsonRPC(json_rpc::Error),
    Provider(ethers::providers::ProviderError),
    Contract(ethers::contract::ContractError<ethers::providers::Provider<ethers::providers::Http>>),
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boxed(error) => write!(f, "{}", error),
            Self::JsonRPC(error) => write!(f, "{}", error),
            Self::Provider(error) => write!(f, "{}", error),
            Self::Contract(error) => write!(f, "{}", error),
        }
    }
}

impl From<json_rpc::Error> for ErrorSource {
    fn from(value: json_rpc::Error) -> Self {
        Self::JsonRPC(value)
    }
}

impl From<ethers::providers::ProviderError> for ErrorSource {
    fn from(value: ethers::providers::ProviderError) -> Self {
        Self::Provider(value)
    }
}

impl From<ethers::contract::ContractError<ethers::providers::Provider<ethers::providers::Http>>>
    for ErrorSource
{
    fn from(
        value: ethers::contract::ContractError<
            ethers::providers::Provider<ethers::providers::Http>,
        >,
    ) -> Self {
        Self::Contract(value)
    }
}

pub struct Error {
    kind: ErrorKind,
    source: ErrorSource,
}

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
}
