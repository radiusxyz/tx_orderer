use std::panic::Location;

pub trait WrapError {
    type Output;

    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug;
}

impl<T> WrapError for Result<T, Error> {
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(error.push_context(context)),
        }
    }
}

impl<T> WrapError for Option<T> {
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug,
    {
        match self {
            Some(value) => Ok(value),
            None => Err(Error {
                backtrace: vec![ErrorFrame::new(context)],
                source: ErrorKind::NoneType,
            }),
        }
    }
}

pub struct Error {
    backtrace: Vec<ErrorFrame>,
    source: ErrorKind,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_verbose(f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_succint(f)
    }
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self {
            backtrace: Vec::new(),
            source: ErrorKind::Custom(value.to_string()),
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self {
            backtrace: Vec::new(),
            source: ErrorKind::Custom(value),
        }
    }
}

impl Error {
    pub fn new<E>(error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            backtrace: Vec::new(),
            source: ErrorKind::Boxed(Box::new(error)),
        }
    }

    pub fn is_none_type(&self) -> bool {
        match &self.source {
            ErrorKind::NoneType => true,
            _others => false,
        }
    }

    #[track_caller]
    fn push_context<C>(mut self, context: C) -> Self
    where
        C: std::fmt::Debug,
    {
        self.backtrace.push(ErrorFrame::new(context));
        self
    }

    fn fmt_verbose(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error:")?;
        for error_frame in self.backtrace.iter().rev() {
            writeln!(f, "\t{:?}", error_frame)?;
        }
        writeln!(f, "Caused by:")?;
        writeln!(f, "\t{}", self.source)?;
        Ok(())
    }

    fn fmt_succint(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.backtrace.first() {
            Some(error) => {
                // # Safety
                // Safe to call unwrap() on self.bactrace.last() because it is guaranteed to exist.
                let top_level_caller = self.backtrace.last().unwrap();
                write!(
                    f,
                    "{} at {}:{}",
                    error,
                    top_level_caller.location.file(),
                    top_level_caller.location.line(),
                )
            }
            None => write!(f, "{}", self.source),
        }
    }
}

struct ErrorFrame {
    location: Location<'static>,
    message: String,
}

impl std::fmt::Debug for ErrorFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.message,
            self.location.file(),
            self.location.line(),
        )
    }
}

impl std::fmt::Display for ErrorFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ErrorFrame {
    #[track_caller]
    pub fn new<C>(context: C) -> Self
    where
        C: std::fmt::Debug,
    {
        Self {
            location: *Location::caller(),
            message: format!("{:?}", context),
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
            Self::NoneType => write!(f, "The value returned None"),
        }
    }
}
