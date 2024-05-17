use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};

pub struct Error {
    kind: ErrorKind,
    source: Option<Box<dyn std::error::Error>>,
}

unsafe impl Send for Error {}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) => write!(f, "{}", source),
            None => write!(f, "{}", self.kind),
        }
    }
}

impl std::error::Error for Error {}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self {
            kind: value,
            source: None,
        }
    }
}

impl Into<String> for Error {
    fn into(self) -> String {
        match self.source {
            Some(source) => source.to_string(),
            None => self.kind.to_string(),
        }
    }
}

impl Into<ErrorObjectOwned> for Error {
    fn into(self) -> ErrorObjectOwned {
        ErrorObjectOwned::owned::<String>(ErrorCode::InternalError.code(), self, None)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self {
            kind: ErrorKind::Custom(value.to_string()),
            source: None,
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self {
            kind: ErrorKind::Custom(value),
            source: None,
        }
    }
}

impl Error {
    pub fn new<E>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            kind: ErrorKind::External,
            source: Some(Box::new(error)),
        }
    }
}

pub enum ErrorKind {
    Custom(String),
    NoneType,
    External,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(error) => write!(f, "{}", error),
            Self::NoneType => write!(f, "The value returned None"),
            _else => write!(f, ""),
        }
    }
}
