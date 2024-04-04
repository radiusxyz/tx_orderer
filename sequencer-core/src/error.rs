use std::panic::Location;

pub trait WrapError {
    type Output;

    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug;
}

impl<T, E> WrapError for Result<T, E>
where
    E: std::error::Error + 'static,
{
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::new(error, context)),
        }
    }
}

pub struct Error {
    source: ErrorKind,
    location: Location<'static>,
    context: Option<String>,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.context {
            Some(context) => write!(
                f,
                "{} at {}:{}\n\t{}",
                self.source,
                self.location.file(),
                self.location.line(),
                context
            ),
            None => write!(
                f,
                "{} at {}:{}",
                self.source,
                self.location.file(),
                self.location.line()
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<&'static str> for Error {
    #[track_caller]
    fn from(value: &'static str) -> Self {
        Self {
            source: ErrorKind::StaticStr(value),
            location: *Location::caller(),
            context: None,
        }
    }
}

impl From<String> for Error {
    #[track_caller]
    fn from(value: String) -> Self {
        Self {
            source: ErrorKind::String(value),
            location: *Location::caller(),
            context: None,
        }
    }
}

impl Error {
    #[track_caller]
    pub fn new<E, C>(error: E, context: C) -> Self
    where
        E: std::error::Error + 'static,
        C: std::fmt::Debug,
    {
        Self {
            source: ErrorKind::Boxed(Box::new(error)),
            location: *Location::caller(),
            context: Some(format!("{:?}", context)),
        }
    }
}

pub enum ErrorKind {
    Boxed(Box<dyn std::error::Error>),
    StaticStr(&'static str),
    String(String),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boxed(error) => write!(f, "{}", error),
            Self::StaticStr(error) => write!(f, "{}", error),
            Self::String(error) => write!(f, "{}", error),
        }
    }
}
