use std::{
    fmt::{Arguments, Write},
    panic::Location,
};

use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};

pub trait WrapError {
    type Output;

    fn wrap(self, context: impl Into<ErrorContext>) -> Self::Output;
}

impl<T, E> WrapError for Result<T, E>
where
    E: std::error::Error + 'static,
{
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap(self, context: impl Into<ErrorContext>) -> Self::Output {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::new_with_context(error, context)),
        }
    }
}

impl<T> WrapError for Option<T> {
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap(self, context: impl Into<ErrorContext>) -> Self::Output {
        match self {
            Some(value) => Ok(value),
            None => Err(Error::none_type(context)),
        }
    }
}

pub struct Error {
    location: Location<'static>,
    source: ErrorKind,
    context: ErrorContext,
}

unsafe impl Send for Error {}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\nError at {}:{}:",
            self.location.file(),
            self.location.line()
        )?;
        writeln!(f, "\t{}", self.source)?;
        write!(f, "{}", self.context)?;
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
            context: ErrorContext::empty(),
        }
    }
}

impl From<String> for Error {
    #[track_caller]
    fn from(value: String) -> Self {
        Self {
            location: *Location::caller(),
            source: ErrorKind::Custom(value),
            context: ErrorContext::empty(),
        }
    }
}

impl Into<ErrorObjectOwned> for Error {
    fn into(self) -> ErrorObjectOwned {
        ErrorObjectOwned::owned::<String>(
            ErrorCode::InternalError.code(),
            self.source,
            Some(self.context.as_string()),
        )
    }
}

impl Error {
    #[track_caller]
    pub fn new<E, C>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            location: *Location::caller(),
            source: ErrorKind::Boxed(Box::new(error)),
            context: ErrorContext::empty(),
        }
    }

    #[track_caller]
    pub fn new_with_context<E, C>(error: E, context: C) -> Self
    where
        E: std::error::Error + 'static,
        C: Into<ErrorContext>,
    {
        Self {
            location: *Location::caller(),
            source: ErrorKind::Boxed(Box::new(error)),
            context: context.into(),
        }
    }

    #[track_caller]
    pub fn none_type<C>(context: C) -> Self
    where
        C: Into<ErrorContext>,
    {
        Self {
            location: *Location::caller(),
            source: ErrorKind::NoneType,
            context: context.into(),
        }
    }

    pub fn is_none_type(&self) -> bool {
        match &self.source {
            ErrorKind::NoneType => true,
            _others => false,
        }
    }
}

enum ErrorKind {
    Boxed(Box<dyn std::error::Error>),
    Custom(String),
    NoneType,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boxed(error) => write!(f, "{}", error),
            Self::Custom(error) => write!(f, "{}", error),
            Self::NoneType => write!(f, "Value returned None"),
        }
    }
}

impl Into<String> for ErrorKind {
    fn into(self) -> String {
        match self {
            Self::Boxed(error) => format!("{}", error),
            Self::Custom(error) => error,
            Self::NoneType => String::from("Value returned None"),
        }
    }
}

pub struct ErrorContext(String);

impl std::fmt::Debug for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.is_empty() {
            true => Ok(()),
            false => writeln!(f, "ErrorContext: {}", self.0),
        }
    }
}

impl std::fmt::Write for ErrorContext {
    fn write_fmt(&mut self, args: Arguments<'_>) -> std::fmt::Result {
        self.0.write_fmt(args)
    }

    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_str(s)
    }
}

impl From<Arguments<'_>> for ErrorContext {
    fn from(value: Arguments<'_>) -> Self {
        let mut context = Self::empty();
        context.write_fmt(value).unwrap();
        context
    }
}

impl From<&str> for ErrorContext {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ErrorContext {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ErrorContext {
    pub fn empty() -> Self {
        Self(String::new())
    }

    pub fn as_string(self) -> String {
        self.0
    }
}
