use std::panic::Location;

use jsonrpsee::types::ErrorCode;

use crate::jsonrpsee::types::ErrorObjectOwned;

pub trait WrapError {
    type Output;

    fn wrap(self, context: String) -> Self::Output;
}

impl<T, E> WrapError for Result<T, E>
where
    E: std::error::Error + 'static,
{
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap(self, context: String) -> Self::Output {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::boxed(error, Some(context))),
        }
    }
}

impl<T> WrapError for Option<T> {
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap(self, context: String) -> Self::Output {
        match self {
            Some(value) => Ok(value),
            None => Err(Error::none_type(Some(context))),
        }
    }
}

pub struct Error {
    location: Location<'static>,
    source: ErrorKind,
    context: Option<String>,
}

unsafe impl Send for Error {}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error:")?;
        writeln!(
            f,
            "\t{} at {}:{}",
            self.source,
            self.location.file(),
            self.location.line(),
        )?;
        writeln!(f, "Context:")?;

        match &self.context {
            Some(context) => writeln!(f, "{}", context)?,
            None => writeln!(f, "None")?,
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    #[track_caller]
    fn from(value: &str) -> Self {
        Self {
            location: *Location::caller(),
            source: ErrorKind::Custom(value.to_string()),
            context: None,
        }
    }
}

impl From<String> for Error {
    #[track_caller]
    fn from(value: String) -> Self {
        Self {
            location: *Location::caller(),
            source: ErrorKind::Custom(value),
            context: None,
        }
    }
}

impl Into<ErrorObjectOwned> for Error {
    fn into(self) -> ErrorObjectOwned {
        match self.context {
            Some(context) => ErrorObjectOwned::owned::<String>(
                ErrorCode::InternalError.code(),
                self.source,
                Some(context),
            ),
            None => ErrorObjectOwned::owned::<String>(
                ErrorCode::InternalError.code(),
                self.source,
                None,
            ),
        }
    }
}

impl Error {
    #[track_caller]
    pub fn boxed<E>(error: E, context: Option<String>) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            location: *Location::caller(),
            source: ErrorKind::Boxed(Box::new(error)),
            context,
        }
    }

    #[track_caller]
    pub fn none_type(context: Option<String>) -> Self {
        Self {
            location: *Location::caller(),
            source: ErrorKind::NoneType,
            context,
        }
    }
}

enum ErrorKind {
    Boxed(Box<dyn std::error::Error>),
    Custom(String),
    NoneType,
}

impl std::fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boxed(error) => write!(f, "{}", error),
            Self::Custom(error) => write!(f, "{}", error),
            Self::NoneType => write!(f, "NoneType Error"),
        }
    }
}

impl Into<String> for ErrorKind {
    fn into(self) -> String {
        format!("{}", self)
    }
}
