use std::panic::Location;

pub trait WrapError {
    type Output;

    fn wrap()
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
    pub fn new<E>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            source: ErrorKind::Boxed(Box::new(error)),
            location: *Location::caller(),
            context: None,
        }
    }

    pub fn with_context<C>(mut self, context: C) -> Self
    where
        C: std::fmt::Debug,
    {
        self.context = Some(format!("{:?}", context));
        self
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

#[test]
fn works() {
    std::fs::read_to_string("no_file")
        .map_err(Error::new)
        .unwrap();
}
