use std::panic::Location;

pub trait WrapError {
    type Output;

    fn wrap(self, caller: &'static str) -> Self::Output;

    fn wrap_context<C>(self, caller: &'static str, context: C) -> Self::Output
    where
        C: std::fmt::Debug;
}

impl<T, E> WrapError for Result<T, E>
where
    E: std::error::Error + 'static,
{
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap(self, caller: &'static str) -> Self::Output {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::new(caller, error)),
        }
    }

    #[track_caller]
    fn wrap_context<C>(self, caller: &'static str, context: C) -> Self::Output
    where
        C: std::fmt::Debug,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::new_with_context(caller, context, error)),
        }
    }
}

impl<T> WrapError for Option<T> {
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap(self, caller: &'static str) -> Self::Output {
        match self {
            Some(value) => Ok(value),
            None => Err(Error::new(caller, ErrorKind::NoneType)),
        }
    }

    #[track_caller]
    fn wrap_context<C>(self, caller: &'static str, context: C) -> Self::Output
    where
        C: std::fmt::Debug,
    {
        match self {
            Some(value) => Ok(value),
            None => Err(Error::new_with_context(
                caller,
                context,
                ErrorKind::NoneType,
            )),
        }
    }
}

pub struct Error {
    operation: &'static str,
    location: Location<'static>,
    context: Option<String>,
    source: ErrorKind,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_caller(f)?;
        self.fmt_context(f)?;
        self.fmt_source(f)?;
        Ok(())
    }
}

impl std::error::Error for Error {}

impl Error {
    fn fmt_caller(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} at {}:{}",
            self.operation,
            self.location.file(),
            self.location.line()
        )
    }

    fn fmt_context(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(context) = &self.context {
            writeln!(f, "\t{}", context)?;
        }
        Ok(())
    }

    fn fmt_source(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }

    #[track_caller]
    pub fn new<E>(operation: &'static str, error: E) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self {
            operation,
            location: *Location::caller(),
            context: None,
            source: ErrorKind::Boxed(Box::new(error)),
        }
    }

    #[track_caller]
    pub fn new_with_context<C, E>(operation: &'static str, context: C, error: E) -> Self
    where
        C: std::fmt::Debug,
        E: std::error::Error + 'static,
    {
        Self {
            operation,
            location: *Location::caller(),
            context: Some(format!("{:?}", context)),
            source: ErrorKind::Boxed(Box::new(error)),
        }
    }
}

pub enum ErrorKind {
    Boxed(Box<dyn std::error::Error>),
    StaticStr(&'static str),
    String(String),
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
            Self::StaticStr(error) => write!(f, "{}", error),
            Self::String(error) => write!(f, "{}", error),
            Self::NoneType => write!(f, "NoneType Error"),
        }
    }
}

impl std::error::Error for ErrorKind {}

#[test]
fn works() {
    fn f1() -> Result<(), Error> {
        f2().wrap(crate::caller!(f1()))
    }

    fn f2() -> Result<(), Error> {
        f3().wrap(crate::caller!(f2()))
    }

    fn f3() -> Result<(), Error> {
        std::fs::read_to_string("no_file").wrap_context(crate::caller!(f3()), "no_file")?;
        Ok(())
    }

    f1().unwrap_or_else(|error| println!("{}", error));
}
