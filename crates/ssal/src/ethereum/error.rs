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
}

pub struct Error {
    kind: ErrorKind,
    source: Option<Box<dyn std::error::Error>>,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(error) => write!(f, "{:?}: {}", self.kind, error),
            None => write!(f, "{:?}", self.kind),
        }
    }
}

impl std::error::Error for Error {}

impl<E> From<(ErrorKind, E)> for Error
where
    E: std::error::Error + 'static,
{
    fn from(value: (ErrorKind, E)) -> Self {
        Self {
            kind: value.0,
            source: Some(value.1.into()),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self {
            kind: value,
            source: None,
        }
    }
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}
