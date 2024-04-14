mod context;
mod database;

use context::Context;
pub use database::DatabaseError;

pub trait WrapError {
    type Output;

    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug;
}

impl<T> WrapError for Result<T, ErrorKind> {
    type Output = Result<T, Error>;

    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::new(context, error)),
        }
    }
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
            Err(mut error) => {
                error.push_context(context);
                Err(error)
            }
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
            None => Err(Error::new(context, ErrorKind::NoneType)),
        }
    }
}

pub struct Error {
    backtrace: Vec<Context>,
    source: ErrorKind,
}

unsafe impl Send for Error {}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error:")?;
        for context in self.backtrace.iter().rev() {
            writeln!(f, "\t{:?}", context)?;
        }
        writeln!(f, "Caused by: \n\t{}", self.source)?;
        Ok(())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.backtrace.first() {
            Some(context) => write!(f, "{}", context),
            None => write!(f, "{}", self.source),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    #[track_caller]
    pub fn new<C>(context: C, source: ErrorKind) -> Self
    where
        C: std::fmt::Debug,
    {
        Self {
            backtrace: vec![Context::new(context)],
            source,
        }
    }

    #[track_caller]
    pub fn push_context<C>(&mut self, context: C)
    where
        C: std::fmt::Debug,
    {
        self.backtrace.push(Context::new(context))
    }

    pub fn is_none_type(&self) -> bool {
        match &self.source {
            ErrorKind::NoneType => true,
            _others => false,
        }
    }
}

pub enum ErrorKind {
    Boxed(Box<dyn std::error::Error>),
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
            Self::Boxed(source) => write!(f, "{}", source),
            Self::NoneType => write!(f, "The value returned None"),
        }
    }
}

impl<E> From<E> for ErrorKind
where
    E: std::error::Error + 'static,
{
    fn from(value: E) -> Self {
        ErrorKind::Boxed(Box::new(value))
    }
}
