use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    BuildClient,
    RpcRequest,
    RegisterRpcMethod,
    ParseParameter,
    RpcMiddleware,
    BuildServer,
}

enum ErrorSource {
    RpcClient(jsonrpsee::core::ClientError),
    RpcMethod(jsonrpsee::core::RegisterMethodError),
    RpcMiddleware(jsonrpsee::server::middleware::http::InvalidPath),
    RpcServer(std::io::Error),
    Custom(String),
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RpcClient(error) => write!(f, "{}", error),
            Self::RpcMethod(error) => write!(f, "{}", error),
            Self::RpcMiddleware(error) => write!(f, "{}", error),
            Self::RpcServer(error) => write!(f, "{}", error),
            Self::Custom(error) => write!(f, "{}", error),
        }
    }
}

impl From<jsonrpsee::core::ClientError> for ErrorSource {
    fn from(value: jsonrpsee::core::ClientError) -> Self {
        Self::RpcClient(value)
    }
}

impl From<jsonrpsee::core::RegisterMethodError> for ErrorSource {
    fn from(value: jsonrpsee::core::RegisterMethodError) -> Self {
        Self::RpcMethod(value)
    }
}

impl From<jsonrpsee::server::middleware::http::InvalidPath> for ErrorSource {
    fn from(value: jsonrpsee::server::middleware::http::InvalidPath) -> Self {
        Self::RpcMiddleware(value)
    }
}

impl From<std::io::Error> for ErrorSource {
    fn from(value: std::io::Error) -> Self {
        Self::RpcServer(value)
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

pub struct RpcError(Box<dyn std::error::Error>);

impl<E> From<E> for RpcError
where
    E: std::error::Error + 'static,
{
    fn from(value: E) -> Self {
        Self(Box::new(value))
    }
}

impl Into<String> for RpcError {
    fn into(self) -> String {
        self.0.to_string()
    }
}

impl Into<ErrorObjectOwned> for RpcError {
    fn into(self) -> ErrorObjectOwned {
        ErrorObjectOwned::owned::<u8>(ErrorCode::InternalError.code(), self, None)
    }
}
